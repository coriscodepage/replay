use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ffi::c_void;
use std::ops::{Add, Sub};
use std::ptr::null_mut;
use std::sync::{Mutex, Once};

use crate::call::Call;
use crate::value_structure::{self, Blob, None, Pointer, Value};

#[derive(Debug, Clone)]
pub struct Region {
    pub buffer: *mut u8,
    pub size: usize,
    pub dimensions: u32,
    pub trace_pitch: i32,
    pub real_pitch: i32,
}

impl Region {
    pub fn new(buffer: *mut u8, size: usize) -> Self {
        Region {
            buffer,
            size,
            dimensions: 0,
            trace_pitch: 0,
            real_pitch: 0,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Range {
    pub ptr: *mut u8,
    pub len: usize,
    pub dims: u32,
    pub trace_pitch: i32,
    pub real_pitch: i32,
}

static mut REGION_MAP: Option<RefCell<BTreeMap<usize, Region>>> = None;
static INIT: Once = Once::new();
static mut OBJ_MAP: Option<RefCell<HashMap<usize, *mut c_void>>> = None;

#[allow(static_mut_refs)]
fn region_map() -> &'static RefCell<BTreeMap<usize, Region>> {
    unsafe {
        INIT.call_once(|| {
            REGION_MAP = Some(RefCell::new(BTreeMap::new()));
        });
        REGION_MAP.as_ref().unwrap()
    }
}

#[allow(static_mut_refs)]
fn obj_map() -> &'static RefCell<HashMap<usize, *mut c_void>> {
    unsafe {
        if OBJ_MAP.is_none() {
            OBJ_MAP = Some(RefCell::new(HashMap::new()));
        }
        OBJ_MAP.as_ref().unwrap()
    }
}

fn contains((addr, region): (&usize, &Region), address: usize) -> bool {
    *addr <= address && (addr + region.size) > address
}

fn intersects((addr, region): (&usize, &Region), start: usize, size: usize) -> bool {
    let it_start = *addr;
    let it_stop = it_start + region.size;
    let stop = start + size;
    it_start < stop && start < it_stop
}

pub fn add_region(address: usize, buffer: *mut u8, size: usize) {
    if address == 0 {
        panic!("Expected a pointer got a nullptr");
    }

    let mut map = region_map().borrow_mut();

    // let overlaps: Vec<_> = map
    //     .range(..=address + size - 1)
    //     .filter(|(k, _)| intersects((*k, &map[k]), address, size))
    //     .collect();

    // for (addr, reg) in overlaps {
    //     eprintln!(
    //         "warning: new region 0x{:x}-0x{:x} intersects existing 0x{:x}-0x{:x}",
    //         address,
    //         address + size,
    //         addr,
    //         addr + reg.size
    //     );
    // }

    map.insert(address, Region::new(buffer, size));
}

pub fn del_region(address: usize) {
    let mut map = region_map().borrow_mut();
    assert!(map.remove(&address).is_some());
}

pub fn del_region_by_pointer(ptr: *mut u8) {
    let mut map = region_map().borrow_mut();
    let addr = map
        .iter()
        .find_map(|(k, region)| if region.buffer == ptr { Some(*k) } else { None });
    assert!(map.remove(&addr.unwrap()).is_some());
}

pub fn set_region_pitch(address: usize, dims: u32, trace_pitch: i32, real_pitch: i32) {
    let mut map = region_map().borrow_mut();
    let region = map
        .get_mut(&lookup_region_key(address).expect("Region not found"))
        .unwrap();
    region.dimensions = dims;
    region.trace_pitch = trace_pitch;
    region.real_pitch = real_pitch;
}

pub fn lookup_region_key(address: usize) -> Option<usize> {
    let map = region_map().borrow_mut();
    let mut keys: Vec<&usize> = map.keys().collect();
    keys.sort();

    for &k in keys.iter().rev() {
        if contains((&k, &map[&k]), address) {
            return Some(*k);
        }
    }
    None
}

pub fn lookup_address(address: usize, range: &mut Range) {
    let map = region_map().borrow_mut();
    if let Some(key) = lookup_region_key(address) {
        let region = &map[&key];
        let offset = address - key;
        assert!(offset < region.size);

        range.ptr = unsafe { region.buffer.add(offset as usize) };
        range.len = region.size - offset;
        range.dims = region.dimensions;
        range.trace_pitch = region.trace_pitch;
        range.real_pitch = region.real_pitch;
        return;
    }

    range.ptr = address as *mut u8;
    range.len = 0;
    range.dims = 0;
    range.trace_pitch = 0;
    range.real_pitch = 0;
}

pub struct Translator<'a> {
    bind: bool,
    range: &'a mut Range,
}

enum Translatable {
    None(None),
    Blob(Blob),
    Pointer(Pointer),
}

impl<'a> Translator<'a> {
    pub fn new(bind: bool, range: &'a mut Range) -> Self {
        range.ptr = null_mut();
        range.len = 0;
        range.dims = 0;
        range.trace_pitch = 0;
        range.real_pitch = 0;
        Self { bind, range }
    }

    pub fn apply(&mut self, value: Translatable) {
        match value {
            Translatable::None(_) => {
                self.range.ptr = null_mut();
                self.range.len = 0;
                self.range.dims = 0;
            }
            Translatable::Blob(mut blob) => {
                blob.bound = self.bind;
                self.range.ptr = blob.to_pointer().unwrap() as *mut u8;
                self.range.len = blob.size;
                self.range.dims = 0;
            }
            Translatable::Pointer(p) => {
                lookup_address(p.value as usize, self.range);
            }
        }
    }
}

pub fn to_range(value: &dyn Value, range: &mut Range) {
    if let Some(_) = value.as_any().downcast_ref::<value_structure::None>() {
        Translator::new(false, range).apply(Translatable::None(value_structure::None {}));
    } else if let Some(pointer_type) = value.as_any().downcast_ref::<value_structure::Pointer>() {
        Translator::new(false, range).apply(Translatable::Pointer(value_structure::Pointer {
            value: pointer_type.value,
        }));
    } else if let Some(blob_type) = value.as_any().downcast_ref::<value_structure::Blob>() {
        Translator::new(false, range).apply(Translatable::Blob(value_structure::Blob {
            size: blob_type.size,
            buffer: blob_type.buffer.clone(),
            bound: blob_type.bound,
        }));
    }
}

pub fn to_pointer(value: &dyn Value, bind: bool) -> *mut u8 {
    let mut range = Range::default();
    if let Some(_) = value.as_any().downcast_ref::<value_structure::None>() {
        Translator::new(bind, &mut range).apply(Translatable::None(value_structure::None {}));
    } else if let Some(pointer_type) = value.as_any().downcast_ref::<value_structure::Pointer>() {
        Translator::new(bind, &mut range).apply(Translatable::Pointer(value_structure::Pointer {
            value: pointer_type.value,
        }));
    } else if let Some(blob_type) = value.as_any().downcast_ref::<value_structure::Blob>() {
        Translator::new(bind, &mut range).apply(Translatable::Blob(value_structure::Blob {
            size: blob_type.size,
            buffer: blob_type.buffer.clone(),
            bound: blob_type.bound,
        }));
    }
    range.ptr
}

pub fn add_obj(call: &Call, value: &dyn Value, obj: *mut c_void) {
    let address = value.to_pointer();

    if address == None {
        if !obj.is_null() {
            println!("Unexpected non-null object: {:?}", call);
        }
        return;
    } else if let Some(address) = address {
        if obj.is_null() {
            println!("Got null for object 0x{:x}", address as usize);
        }
        let mut map = obj_map().borrow_mut();
        map.insert(address as usize, obj);
    }
}

pub fn del_obj(value: &dyn Value) {
    let address = value.to_pointer();
    if let Some(address) = address {
        let mut map = obj_map().borrow_mut();
        map.remove(&(address as usize));
    }
}

pub fn to_obj_pointer(call: Call, value: &dyn Value) -> *mut c_void {
    let address = value.to_pointer();

    let obj = if let Some(address) = address {
        let map = obj_map().borrow_mut();
        let obj = *map.get(&(address as usize)).unwrap_or(&std::ptr::null_mut());

        if obj.is_null() {
           println!("unknown object 0x{:x}", address as usize);
        }

        obj
    } else {
        std::ptr::null_mut()
    };

    obj
}

pub fn block_on_fence(call: &Call, sync: gl::types::GLsync, flags: gl::types::GLbitfield) -> gl::types::GLenum {
    let mut result: gl::types::GLenum;

    loop {
        result = unsafe { gl::ClientWaitSync(sync, flags, 1000) };
        if result != gl::TIMEOUT_EXPIRED {
            break;
        }
    }

    match result {
        gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => {}
        _ => println!("block_on_fence warn")
    }

    result
}

#[derive(Debug, Default)]
pub struct Map<T>
where
    T: Ord + Copy + Add<Output = T> + Sub<Output = T>,
{
    base: BTreeMap<T, T>,
}

impl<T> Map<T>
where
    T: Ord + Copy + Add<Output = T> + Sub<Output = T>,
{
    pub fn new() -> Self {
        Self {
            base: BTreeMap::new(),
        }
    }

    pub fn get_or_insert(&mut self, key: T) -> T {
        *self.base.entry(key).or_insert(key)
    }

    pub fn get_or_insert_ref(&mut self, key: T) -> &T {
        self.base.entry(key).or_insert(key)
    }

    pub fn find(&self, key: &T) -> Option<&T> {
        self.base.get(key)
    }

    pub fn lookup_uniform_location(&mut self, key: T) -> T {
        let mut it = self.base.range(..=key).next_back();
        match it {
            Some((&k, &v)) => v + (key - k),
            None => {
                self.base.insert(key, key);
                key
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, &T)> {
        self.base.iter()
    }
}

pub fn frame_complete(call: &Call) {}