use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use libloading::{Library, Symbol};

use crate::provider::Provider;

/// Plugin interface exposed by provider dynamic libraries.
pub trait ProviderPlugin: Send + Sync {
    /// Create a provider instance.
    fn create_provider(&self) -> Arc<dyn Provider>;

    /// Version string of the plugin.
    fn version(&self) -> &str;
}

/// Loader interface to allow testing/mocking.
pub trait ProviderLoader: Send + Sync {
    /// Attempt to load a provider by name. Returns Ok(None) when not found.
    fn load(&self, name: &str) -> Result<Option<Arc<dyn Provider>>>;
}

/// Default loader that resolves providers from dynamic libraries on disk.
///
/// Expected filename patterns (per platform):
/// - Windows: `vx_provider_<name>.dll` or `libvx_provider_<name>.dll`
/// - macOS:   `libvx_provider_<name>.dylib`
/// - Linux:   `libvx_provider_<name>.so`
///
/// Plugins must export `_vx_create_plugin` returning `*mut dyn ProviderPlugin`.
#[derive(Default)]
pub struct PluginLoader {
    search_paths: Vec<PathBuf>,
    loaded: RwLock<HashMap<String, LoadedPlugin>>,
}

struct LoadedPlugin {
    #[allow(dead_code)]
    library: Library,
    #[allow(dead_code)]
    plugin: Box<dyn ProviderPlugin>,
    provider: Arc<dyn Provider>,
}

impl PluginLoader {
    /// Create a loader with custom search paths.
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths,
            loaded: RwLock::new(HashMap::new()),
        }
    }

    /// Push an additional search path.
    pub fn push_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    fn find_candidate_paths(&self, name: &str) -> Vec<PathBuf> {
        let ext = std::env::consts::DLL_EXTENSION;
        let mut candidates = Vec::new();
        for dir in &self.search_paths {
            candidates.push(dir.join(format!("vx_provider_{}.{}", name, ext)));
            candidates.push(dir.join(format!("libvx_provider_{}.{}", name, ext)));
        }
        candidates
    }

}

impl ProviderLoader for PluginLoader {

    fn load(&self, name: &str) -> Result<Option<Arc<dyn Provider>>> {
        // Return cached provider if already loaded
        if let Some(entry) = self.loaded.read().unwrap().get(name) {
            return Ok(Some(entry.provider.clone()));
        }

        let candidates = self.find_candidate_paths(name);
        let path = candidates.iter().find(|p| p.exists());
        let Some(path) = path else { return Ok(None); };

        // Actually load plugin
        let (library, plugin): (Library, Box<dyn ProviderPlugin>) = unsafe {
            let lib = Library::new(path).with_context(|| format!("load plugin from {:?}", path))?;
            let create_fn: Symbol<extern "C" fn() -> *mut dyn ProviderPlugin> = lib
                .get(b"_vx_create_plugin")
                .with_context(|| format!("load symbol _vx_create_plugin from {:?}", path))?;
            let raw = create_fn();
            let plugin = Box::from_raw(raw);
            (lib, plugin)
        };

        let provider = plugin.create_provider();
        let version = plugin.version().to_string();

        self.loaded
            .write()
            .unwrap()
            .insert(name.to_string(), LoadedPlugin { library, plugin, provider: provider.clone() });

        tracing::info!(provider = name, version, path = %path.display(), "Loaded provider plugin");
        Ok(Some(provider))
    }
}

/// Helper to build default search paths based on vx path conventions.
pub fn default_plugin_paths(base_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for base in base_dirs {
        paths.push(base.join("plugins"));
    }
    paths
}
