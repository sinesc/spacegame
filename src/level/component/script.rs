/**
 * Script component
 *
 * Entities with this component participate in scripted entity logic.
 * The `u16` value is the script type/behavior ID for dispatch in Itsy code.
 */
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Script(pub u16);
