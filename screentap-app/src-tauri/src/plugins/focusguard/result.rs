
#[derive(Debug)]
pub struct FocusGuardCallbackResult {
    pub invoked_vision_model: bool,
    pub vision_model_success: bool,
    pub vision_model_descriptor: String,
    pub skip_vision_model_reason: Option<SkipVisionModelReason>
}

impl FocusGuardCallbackResult {
    pub fn new() -> FocusGuardCallbackResult {
        FocusGuardCallbackResult {
            invoked_vision_model: false,
            vision_model_success: false,
            vision_model_descriptor: String::new(),
            skip_vision_model_reason: None,
        }
    }
        
}

#[derive(Debug)]
pub enum SkipVisionModelReason {
    NotPrimed,
    InvalidFrontmostApp,
    NotEnoughTimeElapsedSinceAlert,
    PerceptualHashDuplicate,
    DevMode,
    Error
}