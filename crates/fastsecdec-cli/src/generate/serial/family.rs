//! Canonical requested capability set, distinct from physical input identity.
use crate::CliResult;
use fastsecdec::kernel::indexed::ProgramRecipe;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeFamily {
    recipes: Vec<ProgramRecipe>,
    default_recipe: ProgramRecipe,
}
impl RecipeFamily {
    pub fn new(
        recipes: impl IntoIterator<Item = ProgramRecipe>,
        default_recipe: ProgramRecipe,
    ) -> CliResult<Self> {
        let mut selected = BTreeSet::new();
        for recipe in recipes {
            if !selected.insert(recipe) {
                return Err("duplicate requested generation recipe".into());
            }
        }
        if !selected.contains(&default_recipe) {
            return Err("default generation recipe must belong to the requested family".into());
        }
        Ok(Self {
            recipes: selected.into_iter().collect(),
            default_recipe,
        })
    }
    #[cfg(test)]
    pub fn single(recipe: ProgramRecipe) -> Self {
        Self {
            recipes: vec![recipe],
            default_recipe: recipe,
        }
    }
    pub fn recipes(&self) -> &[ProgramRecipe] {
        &self.recipes
    }
    pub fn default_recipe(&self) -> ProgramRecipe {
        self.default_recipe
    }
}
