use std::{collections::HashSet, fmt, str::FromStr};

use serde::{Deserialize, Serialize, Serializer};
use strum::{AsRefStr, EnumIter, IntoStaticStr, VariantNames, ParseError};

/// Describes the modules that should be installed.
///
/// # Example
///
/// Create a [`RpcModuleSelection`] from a selection.
///
/// ```
/// use reth_rpc_server_types::{RethRpcModule, RpcModuleSelection};
/// let config: RpcModuleSelection = vec![RethRpcModule::Eth].into();
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum RpcModuleSelection {
    /// Use _all_ available modules.
    All,
    /// The default modules `eth`, `net`, `web3`
    #[default]
    Standard,
    /// Only use the configured modules.
    Selection(HashSet<RethRpcModule>),
}

// === impl RpcModuleSelection ===

impl RpcModuleSelection {
    /// The standard modules to instantiate by default `eth`, `net`, `web3`
    pub const STANDARD_MODULES: [RethRpcModule; 3] =
        [RethRpcModule::Eth, RethRpcModule::Net, RethRpcModule::Web3];

    /// Returns a selection of [`RethRpcModule`] with all [`RethRpcModule::modules`].
    pub fn all_modules() -> HashSet<RethRpcModule> {
        RethRpcModule::modules().iter().copied().collect()
    }

    /// Returns the [`RpcModuleSelection::STANDARD_MODULES`] as a selection.
    pub fn standard_modules() -> HashSet<RethRpcModule> {
        HashSet::from(Self::STANDARD_MODULES)
    }

    /// All modules that are available by default on IPC.
    ///
    /// By default all modules are available on IPC.
    pub fn default_ipc_modules() -> HashSet<RethRpcModule> {
        Self::all_modules()
    }

    /// Creates a new _unique_ [`RpcModuleSelection::Selection`] from the given items.
    ///
    /// # Note
    ///
    /// This will dedupe the selection and remove duplicates while preserving the order.
    ///
    /// # Example
    ///
    /// Create a selection from the [`RethRpcModule`] string identifiers
    ///
    /// ```
    /// use reth_rpc_server_types::{RethRpcModule, RpcModuleSelection};
    /// let selection = vec!["eth", "admin"];
    /// let config = RpcModuleSelection::try_from_selection(selection).unwrap();
    /// assert_eq!(config, RpcModuleSelection::from([RethRpcModule::Eth, RethRpcModule::Admin]));
    /// ```
    ///
    /// Create a unique selection from the [`RethRpcModule`] string identifiers
    ///
    /// ```
    /// use reth_rpc_server_types::{RethRpcModule, RpcModuleSelection};
    /// let selection = vec!["eth", "admin", "eth", "admin"];
    /// let config = RpcModuleSelection::try_from_selection(selection).unwrap();
    /// assert_eq!(config, RpcModuleSelection::from([RethRpcModule::Eth, RethRpcModule::Admin]));
    /// ```
    pub fn try_from_selection<I, T>(selection: I) -> Result<Self, T::Error>
    where
        I: IntoIterator<Item = T>,
        T: TryInto<RethRpcModule, Error = ParseError>,
    {
        let modules: Result<HashSet<RethRpcModule>, _> = selection.into_iter().map(TryInto::try_into).collect();
        modules.map(Self::Selection)
    }

    /// Returns the number of modules in the selection
    pub fn len(&self) -> usize {
        match self {
            Self::All => RethRpcModule::modules().len(),
            Self::Standard => Self::STANDARD_MODULES.len(),
            Self::Selection(s) => s.len(),
        }
    }

    /// Returns true if no selection is configured
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Selection(sel) => sel.is_empty(),
            _ => false,
        }
    }

    /// Returns an iterator over all configured [`RethRpcModule`]
    pub fn iter_selection(&self) -> Box<dyn Iterator<Item = RethRpcModule> + '_> {
        match self {
            Self::All => Box::new(RethRpcModule::modules().iter().copied()),
            Self::Standard => Box::new(Self::STANDARD_MODULES.iter().copied()),
            Self::Selection(s) => Box::new(s.iter().copied()),
        }
    }

    /// Clones the set of configured [`RethRpcModule`].
    pub fn to_selection(&self) -> HashSet<RethRpcModule> {
        match self {
            Self::All => Self::all_modules(),
            Self::Standard => Self::standard_modules(),
            Self::Selection(s) => s.clone(),
        }
    }

    /// Converts the selection into a [`HashSet`].
    pub fn into_selection(self) -> HashSet<RethRpcModule> {
        match self {
            Self::All => Self::all_modules(),
            Self::Standard => Self::standard_modules(),
            Self::Selection(s) => s,
        }
    }

    /// Returns true if both selections are identical.
    pub fn are_identical(http: Option<&Self>, ws: Option<&Self>) -> bool {
        match (http, ws) {
            // Shortcut for common case to avoid iterating later
            (Some(Self::All), Some(other)) | (Some(other), Some(Self::All)) => {
                other.len() == RethRpcModule::modules().len()
            }

            // If either side is disabled, then the other must be empty
            (Some(some), None) | (None, Some(some)) => some.is_empty(),

            (Some(http), Some(ws)) => http.to_selection() == ws.to_selection(),
            (None, None) => true,
        }
    }
}

impl From<&HashSet<RethRpcModule>> for RpcModuleSelection {
    fn from(s: &HashSet<RethRpcModule>) -> Self {
        Self::from(s.clone())
    }
}

impl From<HashSet<RethRpcModule>> for RpcModuleSelection {
    fn from(s: HashSet<RethRpcModule>) -> Self {
        Self::Selection(s)
    }
}

impl From<&[RethRpcModule]> for RpcModuleSelection {
    fn from(s: &[RethRpcModule]) -> Self {
        Self::Selection(s.iter().copied().collect())
    }
}

impl From<Vec<RethRpcModule>> for RpcModuleSelection {
    fn from(s: Vec<RethRpcModule>) -> Self {
        Self::Selection(s.into_iter().collect())
    }
}

impl<const N: usize> From<[RethRpcModule; N]> for RpcModuleSelection {
    fn from(s: [RethRpcModule; N]) -> Self {
        Self::Selection(s.iter().copied().collect())
    }
}

impl<'a> FromIterator<&'a RethRpcModule> for RpcModuleSelection {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a RethRpcModule>,
    {
        iter.into_iter().copied().collect()
    }
}

impl FromIterator<RethRpcModule> for RpcModuleSelection {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RethRpcModule>,
    {
        Self::Selection(iter.into_iter().collect())
    }
}

impl FromStr for RpcModuleSelection {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::Selection(Default::default()))
        }
        let mut modules = s.split(',').map(str::trim).peekable();
        let first = modules.peek().copied().ok_or(ParseError::VariantNotFound)?;
        match first {
            "all" | "All" => Ok(Self::All),
            "none" | "None" => Ok(Self::Selection(Default::default())),
            _ => Self::try_from_selection(modules),
        }
    }
}

impl fmt::Display for RpcModuleSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.iter_selection().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}

/// Represents RPC modules that are supported by reth
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    AsRefStr,
    IntoStaticStr,
    VariantNames,
    EnumIter,
    Deserialize,
    Serialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum RethRpcModule {
    /// `admin_` module
    Admin,
    /// `debug_` module
    Debug,
    /// `eth_` module, including `eth_callBundle`
    Eth,
    /// `net_` module
    Net,
    /// `trace_` module
    Trace,
    /// `txpool_` module
    Txpool,
    /// `web3_` module
    Web3,
    /// `rpc_` module
    Rpc,
}

impl RethRpcModule {
    /// All variants in a list.
    pub const fn modules() -> [RethRpcModule; 8] {
        [
            RethRpcModule::Admin,
            RethRpcModule::Debug,
            RethRpcModule::Eth,
            RethRpcModule::Net,
            RethRpcModule::Trace,
            RethRpcModule::Txpool,
            RethRpcModule::Web3,
            RethRpcModule::Rpc,
        ]
    }
}

impl fmt::Display for RethRpcModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{s}")
    }
}
