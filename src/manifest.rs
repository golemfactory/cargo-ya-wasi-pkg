use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum MountPoint {
    Ro(String),
    Rw(String),
    Wo(String),
    Private(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    /// Deployment id in url like form.
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<EntryPoint>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entry_points: Vec<EntryPoint>,
    pub runtime: RuntimeType,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mount_points: Vec<MountPoint>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeType {
    Emscripten,
    Wasi,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct EntryPoint {
    pub id: String,
    pub wasm_path: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args_prefix: Vec<String>,
}
