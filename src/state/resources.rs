use std::sync::Arc;

use tracing::warn;

use crate::resources::{LabelSelector, Meta, Spec};

use super::revision::Revision;

/// A data structure that ensures the resources are unique by name, and kept in sorted order for
/// efficient lookup and deterministic ordering.
#[derive(derivative::Derivative)]
#[derivative(PartialEq, Hash)]
#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub struct Resources<T>(imbl::Vector<Arc<T>>);

impl<T> Default for Resources<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Meta + Spec + Clone> Resources<T> {
    /// Insert the resource into the resources set.
    /// Returns whether the insertion succeeded or not.
    ///
    /// Insertion checks that if there is an existing resource by the same name that the uids are
    /// the same and that if the resource version is set that it equals that of the existing
    /// resource.
    ///
    /// It also sets the resource version on the resource before insertion.
    pub fn insert(&mut self, mut res: T, revision: Revision) -> Result<(), ()> {
        if let Some(existing) = self.get_mut(&res.metadata().name) {
            if existing.metadata().uid != res.metadata().uid {
                // TODO: update this to have some conflict-reconciliation thing?
                warn!(
                    "Different uids! {} vs {}",
                    existing.metadata().uid,
                    res.metadata().uid
                );
                Err(())
            } else if !res.metadata().resource_version.is_empty()
                && existing.metadata().resource_version != res.metadata().resource_version
            {
                // ignore changes to resources when resource version is specified but unequal
                warn!("Different resource versions");
                Err(())
            } else {
                // set resource version to mod revision as per https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#concurrency-control-and-consistency
                res.metadata_mut().resource_version = revision.to_string();
                // Update the generation of the resource if the spec (desired state) has changed.
                if res.spec() != existing.spec() {
                    res.metadata_mut().generation += 1;
                }
                *existing = res;
                Ok(())
            }
        } else {
            // set resource version to mod revision as per https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#concurrency-control-and-consistency
            res.metadata_mut().resource_version = revision.to_string();
            let pos = self.get_insertion_pos(&res.metadata().name);
            self.0.insert(pos, Arc::new(res));
            Ok(())
        }
    }

    fn get_insertion_pos(&self, k: &str) -> usize {
        match self
            .0
            .binary_search_by_key(&k.to_owned(), |t| t.metadata().name.clone())
        {
            Ok(p) => p,
            Err(p) => p,
        }
    }

    fn get_pos(&self, k: &str) -> Option<usize> {
        self.0
            .binary_search_by_key(&k.to_owned(), |t| t.metadata().name.clone())
            .ok()
    }

    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.get_pos(name)
            .and_then(|p| self.0.get(p).map(|r| r.as_ref()))
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        self.get_pos(name)
            .and_then(|p| self.0.get_mut(p).map(|r| Arc::make_mut(r)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().map(|r| r.as_ref())
    }

    pub fn remove(&mut self, name: &str) -> Option<T> {
        self.get_pos(name).map(|p| (*self.0.remove(p)).clone())
    }

    pub fn retain(&mut self, f: impl Fn(&T) -> bool) {
        self.0.retain(|r| f(r))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn for_controller<'a>(&'a self, uid: &'a str) -> impl Iterator<Item = &T> + 'a {
        self.0
            .iter()
            .filter(move |t| t.metadata().owner_references.iter().any(|or| or.uid == uid))
            .map(|r| r.as_ref())
    }

    pub fn matching(&self, selector: LabelSelector) -> impl Iterator<Item = &T> {
        self.0
            .iter()
            .filter(move |t| selector.matches(&t.metadata().labels))
            .map(|r| r.as_ref())
    }

    pub fn to_vec(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T: Meta + Spec + Clone> From<Vec<T>> for Resources<T> {
    fn from(value: Vec<T>) -> Self {
        let mut rv = Resources::default();
        for v in value {
            let revision = v
                .metadata()
                .resource_version
                .as_str()
                .try_into()
                .unwrap_or_default();
            rv.insert(v, revision).unwrap();
        }
        rv
    }
}
