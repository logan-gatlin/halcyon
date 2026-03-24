use super::BindingSpec;

pub(crate) fn emit(spec: &BindingSpec) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(spec)
}
