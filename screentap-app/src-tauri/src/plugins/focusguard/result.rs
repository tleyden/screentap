
#[derive(Debug)]
pub struct FocusGuardCallbackResult {
    pub invoked_vision_model: bool,
    pub vision_model_success: bool,
    pub vision_model_descriptor: String,
    pub skip_vision_model_reason: Option<SkipVisionModelReason>,
    pub vision_model_response: Option<String>,
    pub productivity_score: i32
}

impl FocusGuardCallbackResult {
    pub fn new() -> FocusGuardCallbackResult {
        FocusGuardCallbackResult {
            invoked_vision_model: false,
            vision_model_success: false,
            vision_model_descriptor: String::new(),  // TODO: Cleaner way to do this?  Option?
            skip_vision_model_reason: None,
            vision_model_response: None,
            productivity_score: -1
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