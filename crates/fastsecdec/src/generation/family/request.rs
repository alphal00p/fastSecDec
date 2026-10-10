use crate::kernel::indexed::ProgramRecipe;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A canonical generation capability set and its artifact default.
///
/// Resident selection is deliberately separate: a caller can retain a recipe
/// different from the artifact default, or retain none. Neither choice changes
/// the mathematical source identity or which programs the archive contains.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "FamilyFields")]
pub struct RecipeFamily {
    recipes: Vec<ProgramRecipe>,
    default_recipe: ProgramRecipe,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RecipeFamilyError {
    #[error("recipe family requires at least one recipe")]
    Empty,
    #[error("duplicate requested generation recipe: {0:?}")]
    Duplicate(ProgramRecipe),
    #[error("default generation recipe must belong to the requested family")]
    MissingDefault,
    #[error("requested resident recipe must belong to the generated family: {0:?}")]
    MissingResident(ProgramRecipe),
}

impl RecipeFamily {
    /// All native contour capabilities, with deformation opt-in at execution.
    pub fn contour() -> Self {
        Self {
            recipes: vec![
                ProgramRecipe::UndeformedV1,
                ProgramRecipe::FixedV1,
                ProgramRecipe::DynamicPolynomialV1,
                ProgramRecipe::DynamicSignAwareV1,
            ],
            default_recipe: ProgramRecipe::UndeformedV1,
        }
    }
    pub fn new(
        recipes: impl IntoIterator<Item = ProgramRecipe>,
        default_recipe: ProgramRecipe,
    ) -> Result<Self, RecipeFamilyError> {
        let mut selected = BTreeSet::new();
        for recipe in recipes {
            if !selected.insert(recipe) {
                return Err(RecipeFamilyError::Duplicate(recipe));
            }
        }
        if selected.is_empty() {
            return Err(RecipeFamilyError::Empty);
        }
        if !selected.contains(&default_recipe) {
            return Err(RecipeFamilyError::MissingDefault);
        }
        Ok(Self {
            recipes: selected.into_iter().collect(),
            default_recipe,
        })
    }

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

    /// `None` requests no retained resident owner; it does not select the default.
    pub fn validate_resident(
        &self,
        recipe: Option<ProgramRecipe>,
    ) -> Result<(), RecipeFamilyError> {
        if let Some(recipe) = recipe
            && self.recipes.binary_search(&recipe).is_err()
        {
            return Err(RecipeFamilyError::MissingResident(recipe));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilyFields {
    recipes: Vec<ProgramRecipe>,
    default_recipe: ProgramRecipe,
}

impl TryFrom<FamilyFields> for RecipeFamily {
    type Error = RecipeFamilyError;
    fn try_from(fields: FamilyFields) -> Result<Self, Self::Error> {
        Self::new(fields.recipes, fields.default_recipe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_order_default_and_resident_are_distinct() {
        use ProgramRecipe::*;
        let family =
            RecipeFamily::new([DynamicSignAwareV1, UndeformedV1, FixedV1], UndeformedV1).unwrap();
        assert_eq!(
            family.recipes(),
            &[UndeformedV1, FixedV1, DynamicSignAwareV1]
        );
        assert_eq!(family.default_recipe(), UndeformedV1);
        family.validate_resident(None).unwrap();
        family.validate_resident(Some(DynamicSignAwareV1)).unwrap();
        assert_eq!(
            family.validate_resident(Some(DynamicPolynomialV1)),
            Err(RecipeFamilyError::MissingResident(DynamicPolynomialV1))
        );
        let encoded = serde_json::to_value(&family).unwrap();
        assert!(encoded.get("resident_recipe").is_none());
        assert_eq!(
            serde_json::from_value::<RecipeFamily>(encoded).unwrap(),
            family
        );
    }

    #[test]
    fn deserialization_cannot_bypass_capability_admission() {
        use ProgramRecipe::*;
        for (recipes, default, reason) in [
            (vec![], FixedV1, RecipeFamilyError::Empty),
            (
                vec![FixedV1, FixedV1],
                FixedV1,
                RecipeFamilyError::Duplicate(FixedV1),
            ),
            (
                vec![UndeformedV1],
                FixedV1,
                RecipeFamilyError::MissingDefault,
            ),
        ] {
            assert_eq!(RecipeFamily::new(recipes.clone(), default), Err(reason));
            assert!(
                serde_json::from_value::<RecipeFamily>(serde_json::json!({
                    "recipes": recipes, "default_recipe": default
                }))
                .is_err()
            );
        }
        let mut encoded = serde_json::to_value(RecipeFamily::single(FixedV1)).unwrap();
        encoded["resident_recipe"] = serde_json::to_value(FixedV1).unwrap();
        assert!(serde_json::from_value::<RecipeFamily>(encoded).is_err());
    }

    #[test]
    fn existing_cli_journal_v3_family_fields_remain_compatible() {
        // Literal field/recipe spellings written by the preceding private CLI
        // type. Resident selection must never become part of this request ID.
        let old = serde_json::json!({
            "recipes": ["undeformed-v1", "fixed-v1"],
            "default_recipe": "undeformed-v1"
        });
        let family: RecipeFamily = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(serde_json::to_value(family).unwrap(), old);
    }
}
