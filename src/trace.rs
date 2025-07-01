use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::ops::BitOr;
use std::{error::Error, fmt::Display};

use regex::Regex;

use crate::file;
use crate::parser;
use crate::value_structure::Value;

pub enum Event {
    EventEnter,
    EventLeave,
}


impl TryFrom<u8> for Event {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Event::EventEnter),
            1 => Ok(Event::EventLeave),
            _ => Err("Unknown Event type"),
        }
    }
}

#[repr(C)]
#[allow(dead_code)]
pub enum CallDetail {
    CallEnd = 0,
    CallArg,
    CallRet,
    CallThread,
    CallBacktrace,
    CallFlags,
}

#[repr(C)]
#[allow(dead_code)]
pub enum Type {
    TypeNull = 0,
    TypeFalse,
    TypeTrue,
    TypeSint,
    TypeUint,
    TypeFloat,
    TypeDouble,
    TypeString,
    TypeBlob,
    TypeEnum,
    TypeBitmask,
    TypeArray,
    TypeStruct,
    TypeOpaque,
    TypeRepr,
    TypeWstring,
}

#[repr(C)]
enum BacktraceDetail {
    BacktraceEnd = 0,
    BacktraceModule,
    BacktraceFunction,
    BacktraceFilename,
    BacktraceLinenumber,
    BacktraceOffset,
}

#[derive(Debug)]
pub enum FunctionSignatureError {
    ParserError(parser::ParserError),
    SnappyError(file::SnappyError),
}

impl Display for FunctionSignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for FunctionSignatureError {}

impl From<parser::ParserError> for FunctionSignatureError {
    fn from(value: parser::ParserError) -> Self {
        FunctionSignatureError::ParserError(value)
    }
}

impl From<file::SnappyError> for FunctionSignatureError {
    fn from(value: file::SnappyError) -> Self {
        FunctionSignatureError::SnappyError(value)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FunctionSignature {
    pub id: usize,
    pub name: String,
    pub num_args: usize,
    pub arg_names: Vec<String>,
    pub flag: Option<u16>,
    pub state: Option<file::Position>
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EnumValue {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EnumSignature {
    pub id: usize,
    pub num_values: usize,
    pub values: Vec<EnumValue>,
    pub state: Option<file::Position>,
}

#[derive(Default, Debug)]
pub struct Call {
    pub sig: FunctionSignature,
    pub number: usize,
    pub ret: Option<Box<dyn Value>>,
    pub args: Vec<Box<dyn Value>>,
    pub thread_id: u16,
}

enum CallFlags {
    CallFlagRender,
    CallFlagNoSideEffects,
    CallFlagSwapRendertarget,
    CallFlagEndFrame,
    CallFlagVerbose,
    CallFlagMarker,
    CallFlagMarkerPush,
    CallFlagMarkerPop,
    CallFlagSwapbuffers,
}

impl TryInto<u16> for CallFlags {
    type Error = CallError;

    fn try_into(self) -> Result<u16, Self::Error> {
        match self {
            CallFlags::CallFlagRender => Ok(8),
            CallFlags::CallFlagNoSideEffects => Ok(4),
            CallFlags::CallFlagSwapRendertarget => Ok(16),
            CallFlags::CallFlagEndFrame => Ok(32),
            CallFlags::CallFlagVerbose => Ok(128),
            CallFlags::CallFlagMarker => Ok(256),
            CallFlags::CallFlagMarkerPush => Ok(512),
            CallFlags::CallFlagMarkerPop => Ok(1024),
            CallFlags::CallFlagSwapbuffers => Ok(48),
            _ => Err(CallError::ConversionError("Cannot convert Call Flag")),
        }
    }
}

impl BitOr for CallFlags {
    type Output = u16;
    fn bitor(self, rhs: Self) -> Self::Output {
        let lhs: u16 = self.try_into().expect("Cannot convert Call Flag into u16");
        let rhs: u16 = rhs.try_into().expect("Cannot convert Call Flag into u16");
        lhs | rhs
    }
}

#[derive(Debug)]
pub enum CallError {
    RegexError,
    ConversionError(&'static str),
    NoDetailsParsed,
    NoCallAvailable,
}

impl Display for CallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for CallError {}

impl From<regex::Error> for CallError {
    fn from(_: regex::Error) -> Self {
        Self::RegexError
    }
}

impl Call {
    pub fn lookup_call_flag(function_name: &str) -> Result<Option<u16>, CallError> {
        match function_name {
            n if n.starts_with("g") => {
                let re_draw = Regex::new(
                    r"^gl([A-Z][a-z]+)*Draw(Range|Mesh)?(Arrays|Elements)([A-Z][a-zA-Z]*)?$",
                )?;
                let re_misc_draw = Regex::new(
                    r"^gl(
                        CallLists?|
                        Clear|
                        End|
                        DrawPixels|
                        DrawTransformFeedback([A-Z][a-zA-Z]*)?|
                        BlitFramebuffer|
                        Rect[dfis]v?|
                        EvalMesh[0-9]+)[0-9A-Z]*$",
                )?;

                if re_draw.is_match(function_name) || re_misc_draw.is_match(function_name) {
                    return Ok(Some(CallFlags::CallFlagRender.try_into()?));
                }

                let re_fbo = Regex::new(r"^glBindFramebuffer[0-9A-Z]*")?;
                if re_fbo.is_match(function_name) {
                    return Ok(Some(CallFlags::CallFlagSwapRendertarget.try_into()?));
                }

                let re_get = Regex::new(
                    r"^gl(
                        GetFloat|
                        GetInteger|
                        GetVertexAttrib|
                        GetTex(ture)?(Level)?Parameter
                        )\w+$",
                )?;

                if re_get.is_match(function_name) {
                    return Ok(Some(CallFlags::CallFlagNoSideEffects.try_into()?));
                }
            }
            n if n.starts_with("I") => {
                let re_present = Regex::new(r"^IDXGI(Decode)?SwapChain\w*::Present\w*$")?;
                let re_draw = Regex::new(
                    r"^ID3D1(0Device|1DeviceContext)\d*::(Draw\w*|ExecuteCommandList)$",
                )?;
                let re_srt =
                    Regex::new(r"^ID3D1(0Device|1DeviceContext)\d*::OMSetRenderTargets\w*$")?;
                let re_cmql = Regex::new(
                    r"^ID3D1[01]Device\d*::(CheckFormatSupport|CheckMultisampleQualityLevels)$",
                )?;
                if re_draw.is_match(function_name) {
                    return Ok(Some(CallFlags::CallFlagRender.try_into()?));
                }
                if re_srt.is_match(function_name) {
                    return Ok(Some(CallFlags::CallFlagSwapRendertarget.try_into()?));
                }
                if re_present.is_match(function_name) {
                    return Ok(Some(CallFlags::CallFlagEndFrame.try_into()?));
                }
                if re_cmql.is_match(function_name) {
                    return Ok(Some(
                        CallFlags::CallFlagNoSideEffects | CallFlags::CallFlagVerbose,
                    ));
                }
            }
            _ => return Ok(None)
        }
        let length = CALL_FLAG_TABLE.len();
        let mut half = length / 2;
        let mut tail = length - 1;
        let mut head = 0;
        let mut current = CALL_FLAG_TABLE[half];

        while head <= tail {
            match current.0.cmp(function_name) {
                std::cmp::Ordering::Equal => return Ok(Some(current.1)),
                std::cmp::Ordering::Less => head = half + 1,
                std::cmp::Ordering::Greater => tail = half - 1,
            }
            half = (tail + head) / 2;
            current = CALL_FLAG_TABLE[half];
        }
        return Ok(None);
    }
}

impl FunctionSignature {}

static CALL_FLAG_TABLE: [(&'static str, u16); 421] = [
    ("CGLFlushDrawable", 32),
    ("CGLGetCurrentContext", 132),
    ("D3DPERF_BeginEvent", /* 4 | */ 768),
    ("D3DPERF_EndEvent", /* 4 | */ 1280),
    ("D3DPERF_SetMarker", /* 4 | */ 256),
    (
        "ID3D11VideoProcessorEnumerator::CheckVideoProcessorFormat",
        132,
    ),
    ("ID3DUserDefinedAnnotation::BeginEvent", /* 4 | */ 768),
    ("ID3DUserDefinedAnnotation::EndEvent", /* 4 | */ 1280),
    ("ID3DUserDefinedAnnotation::SetMarker", /* 4 | */ 256),
    ("IDirect3D8::CheckDeviceFormat", 132),
    ("IDirect3D8::EnumAdapterModes", 132),
    ("IDirect3D8::GetAdapterModeCount", 132),
    ("IDirect3D8::GetDeviceCaps", 132),
    ("IDirect3D9::CheckDeviceFormat", 132),
    ("IDirect3D9::EnumAdapterModes", 132),
    ("IDirect3D9::GetAdapterModeCount", 132),
    ("IDirect3D9::GetDeviceCaps", 132),
    ("IDirect3D9Ex::CheckDeviceFormat", 132),
    ("IDirect3D9Ex::EnumAdapterModes", 132),
    ("IDirect3D9Ex::GetAdapterModeCount", 132),
    ("IDirect3D9Ex::GetDeviceCaps", 132),
    ("IDirect3DDevice2::DrawIndexedPrimitive", 8),
    ("IDirect3DDevice2::DrawPrimitive", 8),
    ("IDirect3DDevice3::DrawIndexedPrimitive", 8),
    ("IDirect3DDevice3::DrawIndexedPrimitiveStrided", 8),
    ("IDirect3DDevice3::DrawIndexedPrimitiveVB", 8),
    ("IDirect3DDevice3::DrawPrimitive", 8),
    ("IDirect3DDevice3::DrawPrimitiveStrided", 8),
    ("IDirect3DDevice3::DrawPrimitiveVB", 8),
    ("IDirect3DDevice7::Clear", 8),
    ("IDirect3DDevice7::DrawIndexedPrimitive", 8),
    ("IDirect3DDevice7::DrawIndexedPrimitiveStrided", 8),
    ("IDirect3DDevice7::DrawIndexedPrimitiveVB", 8),
    ("IDirect3DDevice7::DrawPrimitive", 8),
    ("IDirect3DDevice7::DrawPrimitiveStrided", 8),
    ("IDirect3DDevice7::DrawPrimitiveVB", 8),
    ("IDirect3DDevice8::Clear", 8),
    ("IDirect3DDevice8::DrawIndexedPrimitive", 8),
    ("IDirect3DDevice8::DrawIndexedPrimitiveUP", 8),
    ("IDirect3DDevice8::DrawPrimitive", 8),
    ("IDirect3DDevice8::DrawPrimitiveUP", 8),
    ("IDirect3DDevice8::DrawRectPatch", 8),
    ("IDirect3DDevice8::DrawTriPatch", 8),
    ("IDirect3DDevice8::GetDeviceCaps", 132),
    ("IDirect3DDevice8::Present", 48),
    ("IDirect3DDevice8::SetRenderTarget", 16),
    ("IDirect3DDevice9::Clear", 8),
    ("IDirect3DDevice9::DrawIndexedPrimitive", 8),
    ("IDirect3DDevice9::DrawIndexedPrimitiveUP", 8),
    ("IDirect3DDevice9::DrawPrimitive", 8),
    ("IDirect3DDevice9::DrawPrimitiveUP", 8),
    ("IDirect3DDevice9::DrawRectPatch", 8),
    ("IDirect3DDevice9::DrawTriPatch", 8),
    ("IDirect3DDevice9::GetDeviceCaps", 132),
    ("IDirect3DDevice9::GetRenderTargetData", 32),
    ("IDirect3DDevice9::Present", 48),
    ("IDirect3DDevice9::SetRenderTarget", 16),
    ("IDirect3DDevice9Ex::Clear", 8),
    ("IDirect3DDevice9Ex::DrawIndexedPrimitive", 8),
    ("IDirect3DDevice9Ex::DrawIndexedPrimitiveUP", 8),
    ("IDirect3DDevice9Ex::DrawPrimitive", 8),
    ("IDirect3DDevice9Ex::DrawPrimitiveUP", 8),
    ("IDirect3DDevice9Ex::DrawRectPatch", 8),
    ("IDirect3DDevice9Ex::DrawTriPatch", 8),
    ("IDirect3DDevice9Ex::GetDeviceCaps", 132),
    ("IDirect3DDevice9Ex::GetRenderTargetData", 32),
    ("IDirect3DDevice9Ex::Present", 48),
    ("IDirect3DDevice9Ex::PresentEx", 48),
    ("IDirect3DDevice9Ex::SetRenderTarget", 16),
    ("IDirect3DSwapChain9::Present", 48),
    ("IDirect3DSwapChain9Ex::Present", 48),
    ("IDirect3DViewport2::Clear", 8),
    ("IDirect3DViewport3::Clear", 8),
    ("IDirect3DViewport3::Clear2", 8),
    ("IDirect3DViewport::Clear", 8),
    ("eglGetConfigAttrib", 128),
    ("eglGetProcAddress", 132),
    ("eglQueryString", 132),
    ("eglSetDamageRegionKHR", 4),
    ("eglSwapBuffers", 48),
    ("eglSwapBuffersWithDamageEXT", 48),
    ("eglSwapBuffersWithDamageKHR", 48),
    ("glAreProgramsResidentNV", 4),
    ("glAreTexturesResident", 4),
    ("glAreTexturesResidentEXT", 4),
    ("glBufferRegionEnabled", 4),
    ("glDebugMessageControl", 4),
    ("glDebugMessageControlARB", 4),
    ("glDebugMessageEnableAMD", 4),
    ("glDebugMessageInsert", 4 | 256),
    ("glDebugMessageInsertAMD", 4 | 256),
    ("glDebugMessageInsertARB", 4 | 256),
    ("glDebugMessageInsertKHR", 4 | 256),
    ("glFrameTerminatorGREMEDY", 32),
    ("glGetActiveAtomicCounterBufferiv", 4),
    ("glGetActiveAttrib", 4),
    ("glGetActiveAttribARB", 4),
    ("glGetActiveSubroutineName", 4),
    ("glGetActiveSubroutineUniformName", 4),
    ("glGetActiveSubroutineUniformiv", 4),
    ("glGetActiveUniform", 4),
    ("glGetActiveUniformARB", 4),
    ("glGetActiveUniformBlockName", 4),
    ("glGetActiveUniformBlockiv", 4),
    ("glGetActiveUniformName", 4),
    ("glGetActiveUniformsiv", 4),
    ("glGetActiveVaryingNV", 4),
    ("glGetArrayObjectfvATI", 4),
    ("glGetArrayObjectivATI", 4),
    ("glGetAttachedObjectsARB", 4),
    ("glGetAttachedShaders", 4),
    ("glGetBooleanIndexedvEXT", 4),
    ("glGetBooleani_v", 4),
    ("glGetBooleanv", 4),
    ("glGetBufferParameteri64v", 4),
    ("glGetBufferParameteriv", 4),
    ("glGetBufferParameterivARB", 4),
    ("glGetBufferParameterui64vNV", 4),
    ("glGetBufferPointerv", 4),
    ("glGetBufferPointervARB", 4),
    ("glGetBufferSubData", 4),
    ("glGetBufferSubDataARB", 4),
    ("glGetClipPlane", 4),
    ("glGetColorTable", 4),
    ("glGetColorTableEXT", 4),
    ("glGetColorTableParameterfv", 4),
    ("glGetColorTableParameterfvEXT", 4),
    ("glGetColorTableParameterfvSGI", 4),
    ("glGetColorTableParameteriv", 4),
    ("glGetColorTableParameterivEXT", 4),
    ("glGetColorTableParameterivSGI", 4),
    ("glGetColorTableSGI", 4),
    ("glGetCombinerInputParameterfvNV", 4),
    ("glGetCombinerInputParameterivNV", 4),
    ("glGetCombinerOutputParameterfvNV", 4),
    ("glGetCombinerOutputParameterivNV", 4),
    ("glGetCombinerStageParameterfvNV", 4),
    ("glGetConvolutionFilterEXT", 4),
    ("glGetConvolutionParameterfv", 4),
    ("glGetConvolutionParameterfvEXT", 4),
    ("glGetConvolutionParameteriv", 4),
    ("glGetConvolutionParameterivEXT", 4),
    ("glGetDetailTexFuncSGIS", 4),
    ("glGetDoubleIndexedvEXT", 4),
    ("glGetDoublei_v", 4),
    ("glGetDoublev", 4),
    ("glGetError", 4), // verbose will be set later for GL_NO_ERROR
    ("glGetFenceivNV", 4),
    ("glGetFinalCombinerInputParameterfvNV", 4),
    ("glGetFinalCombinerInputParameterivNV", 4),
    ("glGetFogFuncSGIS", 4),
    ("glGetFragDataIndex", 4),
    ("glGetFragmentLightfvSGIX", 4),
    ("glGetFragmentLightivSGIX", 4),
    ("glGetFragmentMaterialfvSGIX", 4),
    ("glGetFragmentMaterialivSGIX", 4),
    ("glGetFramebufferAttachmentParameteriv", 4),
    ("glGetFramebufferAttachmentParameterivEXT", 4),
    ("glGetFramebufferParameteriv", 4),
    ("glGetFramebufferParameterivEXT", 4),
    ("glGetGraphicsResetStatusARB", 4),
    ("glGetHandleARB", 4),
    ("glGetHistogramEXT", 4),
    ("glGetHistogramParameterfv", 4),
    ("glGetHistogramParameterfvEXT", 4),
    ("glGetHistogramParameteriv", 4),
    ("glGetHistogramParameterivEXT", 4),
    ("glGetImageTransformParameterfvHP", 4),
    ("glGetImageTransformParameterivHP", 4),
    ("glGetInfoLogARB", 4),
    ("glGetInstrumentsSGIX", 4),
    ("glGetInternalformati64v", 4),
    ("glGetInternalformativ", 4),
    ("glGetInvariantBooleanvEXT", 4),
    ("glGetInvariantFloatvEXT", 4),
    ("glGetInvariantIntegervEXT", 4),
    ("glGetLightfv", 4),
    ("glGetLightiv", 4),
    ("glGetListParameterfvSGIX", 4),
    ("glGetListParameterivSGIX", 4),
    ("glGetLocalConstantBooleanvEXT", 4),
    ("glGetLocalConstantFloatvEXT", 4),
    ("glGetLocalConstantIntegervEXT", 4),
    ("glGetMapAttribParameterfvNV", 4),
    ("glGetMapAttribParameterivNV", 4),
    ("glGetMapControlPointsNV", 4),
    ("glGetMapParameterfvNV", 4),
    ("glGetMapParameterivNV", 4),
    ("glGetMapdv", 4),
    ("glGetMapfv", 4),
    ("glGetMapiv", 4),
    ("glGetMaterialfv", 4),
    ("glGetMaterialiv", 4),
    ("glGetMinmaxEXT", 4),
    ("glGetMinmaxParameterfv", 4),
    ("glGetMinmaxParameterfvEXT", 4),
    ("glGetMinmaxParameteriv", 4),
    ("glGetMinmaxParameterivEXT", 4),
    ("glGetMultiTexEnvfvEXT", 4),
    ("glGetMultiTexEnvivEXT", 4),
    ("glGetMultiTexGendvEXT", 4),
    ("glGetMultiTexGenfvEXT", 4),
    ("glGetMultiTexGenivEXT", 4),
    ("glGetMultiTexLevelParameterfvEXT", 4),
    ("glGetMultiTexLevelParameterivEXT", 4),
    ("glGetMultiTexParameterIivEXT", 4),
    ("glGetMultiTexParameterIuivEXT", 4),
    ("glGetMultiTexParameterfvEXT", 4),
    ("glGetMultiTexParameterivEXT", 4),
    ("glGetMultisamplefv", 4),
    ("glGetMultisamplefvNV", 4),
    ("glGetNamedBufferParameterivEXT", 4),
    ("glGetNamedBufferParameterui64vNV", 4),
    ("glGetNamedBufferPointervEXT", 4),
    ("glGetNamedBufferSubDataEXT", 4),
    ("glGetNamedFramebufferAttachmentParameterivEXT", 4),
    ("glGetNamedFramebufferParameterivEXT", 4),
    ("glGetNamedProgramLocalParameterIivEXT", 4),
    ("glGetNamedProgramLocalParameterIuivEXT", 4),
    ("glGetNamedProgramLocalParameterdvEXT", 4),
    ("glGetNamedProgramLocalParameterfvEXT", 4),
    ("glGetNamedProgramStringEXT", 4),
    ("glGetNamedProgramivEXT", 4),
    ("glGetNamedRenderbufferParameterivEXT", 4),
    ("glGetNamedStringARB", 4),
    ("glGetNamedStringivARB", 4),
    ("glGetObjectBufferfvATI", 4),
    ("glGetObjectBufferivATI", 4),
    ("glGetObjectLabel", 4),
    ("glGetObjectParameterfvARB", 4),
    ("glGetObjectParameterivAPPLE", 4),
    ("glGetObjectParameterivARB", 4),
    ("glGetObjectPtrLabel", 4),
    ("glGetOcclusionQueryivNV", 4),
    ("glGetOcclusionQueryuivNV", 4),
    ("glGetPerfMonitorCounterDataAMD", 4),
    ("glGetPerfMonitorCounterInfoAMD", 4),
    ("glGetPerfMonitorCounterStringAMD", 4),
    ("glGetPerfMonitorCountersAMD", 4),
    ("glGetPerfMonitorGroupStringAMD", 4),
    ("glGetPerfMonitorGroupsAMD", 4),
    ("glGetPixelTexGenParameterfvSGIS", 4),
    ("glGetPixelTexGenParameterivSGIS", 4),
    ("glGetPointerIndexedvEXT", 4),
    ("glGetPointerv", 4),
    ("glGetPointervEXT", 4),
    ("glGetProgramBinary", 4),
    ("glGetProgramEnvParameterIivNV", 4),
    ("glGetProgramEnvParameterIuivNV", 4),
    ("glGetProgramEnvParameterdvARB", 4),
    ("glGetProgramEnvParameterfvARB", 4),
    ("glGetProgramInfoLog", 4),
    ("glGetProgramInterfaceiv", 4),
    ("glGetProgramLocalParameterIivNV", 4),
    ("glGetProgramLocalParameterIuivNV", 4),
    ("glGetProgramLocalParameterdvARB", 4),
    ("glGetProgramLocalParameterfvARB", 4),
    ("glGetProgramNamedParameterdvNV", 4),
    ("glGetProgramNamedParameterfvNV", 4),
    ("glGetProgramParameterdvNV", 4),
    ("glGetProgramParameterfvNV", 4),
    ("glGetProgramPipelineInfoLog", 4),
    ("glGetProgramPipelineiv", 4),
    ("glGetProgramResourceIndex", 4),
    ("glGetProgramResourceLocation", 4),
    ("glGetProgramResourceLocationIndex", 4),
    ("glGetProgramResourceName", 4),
    ("glGetProgramResourceiv", 4),
    ("glGetProgramStageiv", 4),
    ("glGetProgramStringARB", 4),
    ("glGetProgramStringNV", 4),
    ("glGetProgramSubroutineParameteruivNV", 4),
    ("glGetProgramiv", 4),
    ("glGetProgramivARB", 4),
    ("glGetProgramivNV", 4),
    ("glGetQueryIndexediv", 4),
    ("glGetQueryiv", 4),
    ("glGetQueryivARB", 4),
    ("glGetRenderbufferParameteriv", 4),
    ("glGetRenderbufferParameterivEXT", 4),
    ("glGetSamplerParameterIiv", 4),
    ("glGetSamplerParameterIuiv", 4),
    ("glGetSamplerParameterfv", 4),
    ("glGetSamplerParameteriv", 4),
    ("glGetSeparableFilterEXT", 4),
    ("glGetShaderInfoLog", 4),
    ("glGetShaderPrecisionFormat", 4),
    ("glGetShaderSource", 4),
    ("glGetShaderSourceARB", 4),
    ("glGetShaderiv", 4),
    ("glGetSharpenTexFuncSGIS", 4),
    ("glGetString", 132),
    ("glGetStringi", 132),
    ("glGetSynciv", 4),
    ("glGetTexBumpParameterfvATI", 4),
    ("glGetTexBumpParameterivATI", 4),
    ("glGetTexEnvfv", 4),
    ("glGetTexEnviv", 4),
    ("glGetTexFilterFuncSGIS", 4),
    ("glGetTexGendv", 4),
    ("glGetTexGenfv", 4),
    ("glGetTexGeniv", 4),
    ("glGetTrackMatrixivNV", 4),
    ("glGetTransformFeedbackVarying", 4),
    ("glGetTransformFeedbackVaryingEXT", 4),
    ("glGetTransformFeedbackVaryingNV", 4),
    ("glGetUniformIndices", 4),
    ("glGetUniformSubroutineuiv", 4),
    ("glGetUniformdv", 4),
    ("glGetUniformfv", 4),
    ("glGetUniformfvARB", 4),
    ("glGetUniformi64vNV", 4),
    ("glGetUniformiv", 4),
    ("glGetUniformivARB", 4),
    ("glGetUniformui64vNV", 4),
    ("glGetUniformuiv", 4),
    ("glGetUniformuivEXT", 4),
    ("glGetVariantArrayObjectfvATI", 4),
    ("glGetVariantArrayObjectivATI", 4),
    ("glGetVariantBooleanvEXT", 4),
    ("glGetVariantFloatvEXT", 4),
    ("glGetVariantIntegervEXT", 4),
    ("glGetVariantPointervEXT", 4),
    ("glGetVertexArrayIntegeri_vEXT", 4),
    ("glGetVertexArrayIntegervEXT", 4),
    ("glGetVertexArrayPointeri_vEXT", 4),
    ("glGetVertexArrayPointervEXT", 4),
    ("glGetVideoCaptureStreamdvNV", 4),
    ("glGetVideoCaptureStreamfvNV", 4),
    ("glGetVideoCaptureStreamivNV", 4),
    ("glGetVideoCaptureivNV", 4),
    ("glGetVideoi64vNV", 4),
    ("glGetVideoivNV", 4),
    ("glGetVideoui64vNV", 4),
    ("glGetVideouivNV", 4),
    ("glGetnMapdvARB", 4),
    ("glGetnMapfvARB", 4),
    ("glGetnMapivARB", 4),
    ("glGetnUniformdvARB", 4),
    ("glGetnUniformfvARB", 4),
    ("glGetnUniformivARB", 4),
    ("glGetnUniformuivARB", 4),
    ("glInsertEventMarkerEXT", 260),
    ("glIsAsyncMarkerSGIX", 132),
    ("glIsBuffer", 132),
    ("glIsBufferARB", 132),
    ("glIsBufferResidentNV", 132),
    ("glIsEnabled", 132),
    ("glIsEnabledIndexedEXT", 132),
    ("glIsEnabledi", 132),
    ("glIsFenceAPPLE", 132),
    ("glIsFenceNV", 132),
    ("glIsFramebuffer", 132),
    ("glIsFramebufferEXT", 132),
    ("glIsList", 132),
    ("glIsNameAMD", 132),
    ("glIsNamedBufferResidentNV", 132),
    ("glIsNamedStringARB", 132),
    ("glIsObjectBufferATI", 132),
    ("glIsOcclusionQueryNV", 132),
    ("glIsProgram", 132),
    ("glIsProgramARB", 132),
    ("glIsProgramNV", 132),
    ("glIsProgramPipeline", 132),
    ("glIsQuery", 132),
    ("glIsQueryARB", 132),
    ("glIsRenderbuffer", 132),
    ("glIsRenderbufferEXT", 132),
    ("glIsSampler", 132),
    ("glIsShader", 132),
    ("glIsSync", 132),
    ("glIsTexture", 132),
    ("glIsTextureEXT", 132),
    ("glIsTransformFeedback", 132),
    ("glIsTransformFeedbackNV", 132),
    ("glIsVariantEnabledEXT", 132),
    ("glIsVertexArray", 132),
    ("glIsVertexArrayAPPLE", 132),
    ("glIsVertexAttribEnabledAPPLE", 132),
    ("glObjectLabel", 4),
    ("glObjectLabelKHR", 4),
    ("glObjectPtrLabel", 4),
    ("glObjectPtrLabelKHR", 4),
    ("glPopDebugGroup", /* 4 | */ 1280),
    ("glPopDebugGroupKHR", /* 4 | */ 1280),
    ("glPopGroupMarkerEXT", /* 4 | */ 1280),
    ("glPushDebugGroup", /* 4 | */ 768),
    ("glPushDebugGroupKHR", /* 4 | */ 768),
    ("glPushGroupMarkerEXT", /* 4 | */ 768),
    ("glStringMarkerGREMEDY", /* 4 | */ 256),
    ("glXGetClientString", 132),
    ("glXGetConfig", 132),
    ("glXGetCurrentContext", 132),
    ("glXGetCurrentDisplay", 132),
    ("glXGetCurrentDisplayEXT", 132),
    ("glXGetCurrentDrawable", 132),
    ("glXGetCurrentReadDrawable", 132),
    ("glXGetCurrentReadDrawableSGI", 132),
    ("glXGetFBConfigAttrib", 128),
    ("glXGetFBConfigAttribSGIX", 128),
    ("glXGetProcAddress", 132),
    ("glXGetProcAddressARB", 132),
    ("glXIsDirect", 132),
    ("glXQueryExtension", 132),
    ("glXQueryExtensionsString", 132),
    ("glXQueryVersion", 132),
    ("glXSwapBuffers", 48),
    ("glXSwapBuffersMscOML", 48),
    ("wglDescribePixelFormat", 132),
    ("wglGetCurrentContext", 132),
    ("wglGetCurrentDC", 132),
    ("wglGetDefaultProcAddress", 132),
    ("wglGetExtensionsStringARB", 132),
    ("wglGetExtensionsStringEXT", 132),
    ("wglGetPixelFormat", 132),
    ("wglGetPixelFormatAttribivARB", 128),
    ("wglGetPixelFormatAttribivEXT", 128),
    ("wglGetProcAddress", 132),
    ("wglSwapBuffers", 48),
    ("wglSwapLayerBuffers", 48),
    ("wglSwapMultipleBuffers", 48),
    // NOTE: New entries must be sorted alphabetically
];
