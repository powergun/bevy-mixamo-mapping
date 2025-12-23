use super::bone::Bone2D;
use crate::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A 2D skeleton definition, saved as `*.skeleton2d.ron`
///
/// # Invariants
/// - `bones` is non-empty
/// - All bone names are unique
/// - Exactly one bone has `parent = None` (the root)
/// - Parent indices are always < child indices (topological order)
/// - All parent indices are valid (< bones.len())
/// - No circular parent references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skeleton2D {
    /// Unique name for this skeleton
    pub name: String,
    /// Ordered list of bones (index corresponds to bone ID)
    /// Stored in topological order: parents always come before children
    pub bones: Vec<Bone2D>,
}

impl Skeleton2D {
    /// Create a new Skeleton2D with full validation
    ///
    /// # Errors
    /// Returns `ValidationError` if any invariant is violated
    pub fn new(name: impl Into<String>, bones: Vec<Bone2D>) -> Result<Self, ValidationError> {
        let skeleton = Self {
            name: name.into(),
            bones,
        };
        skeleton.validate()?;
        Ok(skeleton)
    }

    /// Validate all skeleton invariants
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_non_empty()?;
        self.validate_unique_names()?;
        self.validate_single_root()?;
        self.validate_parent_indices()?;
        self.validate_topological_order()?;
        self.validate_no_cycles()?;
        self.validate_bone_properties()?;
        Ok(())
    }

    /// Get bone by index
    pub fn get_bone(&self, index: usize) -> Option<&Bone2D> {
        self.bones.get(index)
    }

    /// Get bone by name
    pub fn get_bone_by_name(&self, name: &str) -> Option<(usize, &Bone2D)> {
        self.bones.iter().enumerate().find(|(_, b)| b.name == name)
    }

    /// Get the root bone (the one with parent = None)
    pub fn root_bone(&self) -> &Bone2D {
        // Safe: validate() ensures exactly one root exists
        self.bones.iter().find(|b| b.is_root()).unwrap()
    }

    /// Get root bone index
    pub fn root_bone_index(&self) -> usize {
        self.bones.iter().position(|b| b.is_root()).unwrap()
    }

    /// Get children of a bone
    pub fn children_of(&self, parent_index: usize) -> Vec<usize> {
        self.bones
            .iter()
            .enumerate()
            .filter(|(_, b)| b.parent == Some(parent_index))
            .map(|(i, _)| i)
            .collect()
    }

    /// Build a map from bone name to index
    pub fn name_to_index_map(&self) -> HashMap<&str, usize> {
        self.bones
            .iter()
            .enumerate()
            .map(|(i, b)| (b.name.as_str(), i))
            .collect()
    }

    /// Get bone count
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    // ─────────────────────────────────────────────────────────────
    // Private validation methods
    // ─────────────────────────────────────────────────────────────

    fn validate_non_empty(&self) -> Result<(), ValidationError> {
        if self.bones.is_empty() {
            return Err(ValidationError::EmptyBoneList);
        }
        Ok(())
    }

    fn validate_unique_names(&self) -> Result<(), ValidationError> {
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for (i, bone) in self.bones.iter().enumerate() {
            if let Some(&first_idx) = seen.get(bone.name.as_str()) {
                return Err(ValidationError::DuplicateBoneName {
                    name: bone.name.clone(),
                    first: first_idx,
                    second: i,
                });
            }
            seen.insert(&bone.name, i);
        }
        Ok(())
    }

    fn validate_single_root(&self) -> Result<(), ValidationError> {
        let roots: Vec<_> = self.bones.iter().filter(|b| b.is_root()).collect();
        match roots.len() {
            0 => Err(ValidationError::NoRootBone),
            1 => Ok(()),
            _ => Err(ValidationError::MultipleRootBones {
                first: roots[0].name.clone(),
                second: roots[1].name.clone(),
            }),
        }
    }

    fn validate_parent_indices(&self) -> Result<(), ValidationError> {
        for (i, bone) in self.bones.iter().enumerate() {
            if let Some(parent_idx) = bone.parent {
                if parent_idx >= self.bones.len() {
                    return Err(ValidationError::InvalidParentIndex {
                        name: bone.name.clone(),
                        index: i,
                        parent_index: parent_idx,
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_topological_order(&self) -> Result<(), ValidationError> {
        for (i, bone) in self.bones.iter().enumerate() {
            if let Some(parent_idx) = bone.parent {
                if parent_idx >= i {
                    return Err(ValidationError::ParentAfterChild {
                        name: bone.name.clone(),
                        index: i,
                        parent_index: parent_idx,
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_no_cycles(&self) -> Result<(), ValidationError> {
        for (start_idx, bone) in self.bones.iter().enumerate() {
            let mut visited = HashSet::new();
            let mut current = bone.parent;

            while let Some(idx) = current {
                if !visited.insert(idx) {
                    return Err(ValidationError::CircularParentReference {
                        name: bone.name.clone(),
                    });
                }
                if idx == start_idx {
                    return Err(ValidationError::CircularParentReference {
                        name: bone.name.clone(),
                    });
                }
                current = self.bones.get(idx).and_then(|b| b.parent);
            }
        }
        Ok(())
    }

    fn validate_bone_properties(&self) -> Result<(), ValidationError> {
        for bone in &self.bones {
            if bone.length < 0.0 {
                return Err(ValidationError::InvalidBoneLength {
                    name: bone.name.clone(),
                    length: bone.length,
                });
            }
            if !bone.setup.is_scale_valid() {
                return Err(ValidationError::InvalidBoneScale {
                    name: bone.name.clone(),
                    x: bone.setup.scale.x,
                    y: bone.setup.scale.y,
                });
            }
        }
        Ok(())
    }
}
