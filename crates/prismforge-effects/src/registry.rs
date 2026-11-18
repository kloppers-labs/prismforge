use crate::builtin::BUILTINS;
use crate::definition::EffectDefinition;
use crate::effect::Effect;
use crate::instance::{EffectInstance, InstanceError};

/// Every built-in effect, always in the same order.
pub fn definitions() -> Vec<EffectDefinition> {
    BUILTINS
        .iter()
        .map(|builtin| (builtin.definition)())
        .collect()
}

/// Validates `instance` against its definition and fills defaults.
pub fn prepare(instance: &EffectInstance) -> Result<Effect, InstanceError> {
    for builtin in &BUILTINS {
        let definition = (builtin.definition)();
        if definition.id != instance.effect {
            continue;
        }
        let complete = instance
            .complete(&definition)
            .map_err(|errors| InstanceError::InvalidValues { errors })?;
        let program = (builtin.build)(&complete.checked());
        return Ok(Effect::new(complete, program));
    }
    Err(InstanceError::UnknownEffect {
        effect: instance.effect.clone(),
    })
}
