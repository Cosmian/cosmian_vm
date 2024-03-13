use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum CloudProvider {
    AWS,
    Azure,
    GCP,
}
