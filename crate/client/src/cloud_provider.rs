use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum CloudProvider {
    AWS,
    Azure,
    GCP,
}
