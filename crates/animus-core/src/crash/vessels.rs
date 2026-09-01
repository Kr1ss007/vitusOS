//! Vessels — Subsystem Dependency Graph and Blast Radius Propagation (Part 21.8 of spec).
//!
//! Evaluates failures in any subsystem, computes transitive blast radii via BFS,
//! and executes graceful isolation callbacks rather than bringing down the display server.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tracing::{info, warn};

use crate::event_bus::EventBus;
use crate::events::AEEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VesselState {
    Running,
    Degraded,
    Isolated,
    Dead,
}

pub type IsolationCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct Vessel {
    pub name: String,
    pub state: VesselState,
    pub depends_on: Vec<String>,
    pub on_isolate: Option<IsolationCallback>,
    pub on_restore: Option<IsolationCallback>,
}

pub struct Vessels {
    vessels: Arc<RwLock<HashMap<String, Vessel>>>,
    bus: EventBus,
}

impl Vessels {
    pub fn new(bus: EventBus) -> Self {
        let instance = Self {
            vessels: Arc::new(RwLock::new(HashMap::new())),
            bus,
        };
        instance.register_canonical_vessels();
        instance
    }

    /// Registers a subsystem vessel in the dependency graph.
    pub fn register_vessel(&self, vessel: Vessel) {
        self.vessels.write().insert(vessel.name.clone(), vessel);
    }

    /// Marks a subsystem as Dead, computes its transitive blast radius via BFS, and applies isolation.
    pub fn mark_dead(&self, name: &str) {
        let affected = {
            let mut vessels = self.vessels.write();
            if let Some(v) = vessels.get_mut(name) {
                v.state = VesselState::Dead;
            } else {
                return;
            }
            self.bfs_dependents_locked(&vessels, name)
        };

        warn!("Vessels: Subsystem '{}' died! Computed blast radius: {:?}", name, affected);
        self.apply_isolation(&affected);
        self.bus.publish(AEEvent::SubsystemHealthChanged {
            name: name.to_string(),
            healthy: false,
        });
    }

    /// Marks a subsystem as restored and triggers recovery on dependent vessels.
    pub fn mark_restored(&self, name: &str) {
        let restored_list = {
            let mut vessels = self.vessels.write();
            if let Some(v) = vessels.get_mut(name) {
                v.state = VesselState::Running;
            } else {
                return;
            }
            // Phase 1: Identify isolated vessels whose dependencies are all Running
            let ready_names: Vec<String> = vessels
                .values()
                .filter(|v| {
                    v.state == VesselState::Isolated
                        && v.depends_on.iter().all(|dep| {
                            vessels.get(dep).map(|d| d.state) == Some(VesselState::Running)
                        })
                })
                .map(|v| v.name.clone())
                .collect();

            // Phase 2: Mutate only the matching vessels
            let mut can_restore = Vec::new();
            for ready_name in ready_names {
                if let Some(v) = vessels.get_mut(&ready_name) {
                    v.state = VesselState::Running;
                    can_restore.push(v.clone());
                }
            }
            can_restore
        };

        for v in restored_list {
            if let Some(ref cb) = v.on_restore {
                cb();
            }
            info!("Vessels: Subsystem '{}' restored to Running state.", v.name);
        }

        self.bus.publish(AEEvent::SubsystemHealthChanged {
            name: name.to_string(),
            healthy: true,
        });
    }

    /// Computes the transitive blast radius of a dead subsystem using BFS.
    pub fn blast_radius(&self, name: &str) -> Vec<String> {
        let vessels = self.vessels.read();
        self.bfs_dependents_locked(&vessels, name)
    }

    /// Queries the state of a subsystem.
    pub fn state_of(&self, name: &str) -> VesselState {
        self.vessels.read().get(name).map(|v| v.state).unwrap_or(VesselState::Dead)
    }

    fn bfs_dependents_locked(&self, vessels: &HashMap<String, Vessel>, root: &str) -> Vec<String> {
        // Build reverse dependency adjacency list
        let mut rev_edges: HashMap<String, Vec<String>> = HashMap::new();
        for (name, v) in vessels.iter() {
            for dep in &v.depends_on {
                rev_edges.entry(dep.clone()).or_default().push(name.clone());
            }
        }

        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(root.to_string());
        visited.insert(root.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = rev_edges.get(&current) {
                for dep in dependents {
                    if !visited.contains(dep) {
                        visited.insert(dep.clone());
                        result.push(dep.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        result
    }

    fn apply_isolation(&self, affected: &[String]) {
        let callbacks = {
            let mut vessels = self.vessels.write();
            let mut list = Vec::new();
            for name in affected {
                if let Some(v) = vessels.get_mut(name) {
                    v.state = VesselState::Isolated;
                    if let Some(ref cb) = v.on_isolate {
                        list.push(cb.clone());
                    }
                }
            }
            list
        };

        for cb in callbacks {
            cb();
        }
    }

    fn register_canonical_vessels(&self) {
        self.register_vessel(Vessel {
            name: "Compositor".to_string(),
            state: VesselState::Running,
            depends_on: Vec::new(),
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "VulkanContext".to_string(),
            state: VesselState::Running,
            depends_on: vec!["Compositor".to_string()],
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "GlyphAtlas".to_string(),
            state: VesselState::Running,
            depends_on: vec!["VulkanContext".to_string()],
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "TextRenderer".to_string(),
            state: VesselState::Running,
            depends_on: vec!["GlyphAtlas".to_string()],
            on_isolate: Some(Arc::new(|| {
                warn!("TextRenderer: Isolated. Panel falling back to icon-only mode.");
            })),
            on_restore: Some(Arc::new(|| {
                info!("TextRenderer: Restored text rendering.");
            })),
        });

        self.register_vessel(Vessel {
            name: "MaterialRenderer".to_string(),
            state: VesselState::Running,
            depends_on: vec!["VulkanContext".to_string()],
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "ShadowRenderer".to_string(),
            state: VesselState::Running,
            depends_on: vec!["VulkanContext".to_string()],
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "RenderPipeline".to_string(),
            state: VesselState::Running,
            depends_on: vec![
                "MaterialRenderer".to_string(),
                "ShadowRenderer".to_string(),
                "TextRenderer".to_string(),
            ],
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "Panel".to_string(),
            state: VesselState::Running,
            depends_on: vec!["RenderPipeline".to_string()],
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "Dock".to_string(),
            state: VesselState::Running,
            depends_on: vec!["RenderPipeline".to_string()],
            on_isolate: None,
            on_restore: None,
        });

        self.register_vessel(Vessel {
            name: "SoundEngine".to_string(),
            state: VesselState::Running,
            depends_on: Vec::new(),
            on_isolate: Some(Arc::new(|| {
                warn!("SoundEngine: Isolated. System muted temporarily.");
            })),
            on_restore: Some(Arc::new(|| {
                info!("SoundEngine: Audio pipeline restored.");
            })),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vessels_blast_radius_and_isolation() {
        let bus = EventBus::new();
        let vessels = Vessels::new(bus);

        // Failure of GlyphAtlas should cascade to TextRenderer -> RenderPipeline -> Panel & Dock
        let radius = vessels.blast_radius("GlyphAtlas");
        assert!(radius.contains(&"TextRenderer".to_string()));
        assert!(radius.contains(&"RenderPipeline".to_string()));
        assert!(radius.contains(&"Panel".to_string()));
        assert!(radius.contains(&"Dock".to_string()));
        assert!(!radius.contains(&"SoundEngine".to_string()));

        // Mark GlyphAtlas dead
        vessels.mark_dead("GlyphAtlas");
        assert_eq!(vessels.state_of("GlyphAtlas"), VesselState::Dead);
        assert_eq!(vessels.state_of("TextRenderer"), VesselState::Isolated);
        assert_eq!(vessels.state_of("SoundEngine"), VesselState::Running);

        // Restore GlyphAtlas
        vessels.mark_restored("GlyphAtlas");
        assert_eq!(vessels.state_of("GlyphAtlas"), VesselState::Running);
        assert_eq!(vessels.state_of("TextRenderer"), VesselState::Running);
    }
}
