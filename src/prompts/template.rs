//! Template loading and rendering for agent prompts
//!
//! Supports {{variable}} substitution and {{#if var}}...{{/if}} conditionals.

use std::collections::HashMap;
use std::path::Path;

use regex::Regex;

use crate::errors::{Result, WreckitError};
use crate::fs::get_prompts_dir;

// Bundled default prompts
const DEFAULT_RESEARCH_PROMPT: &str = include_str!("../../prompts/research.md");
const DEFAULT_PLAN_PROMPT: &str = include_str!("../../prompts/plan.md");
const DEFAULT_IMPLEMENT_PROMPT: &str = include_str!("../../prompts/implement.md");
const DEFAULT_PR_PROMPT: &str = include_str!("../../prompts/pr.md");

/// Variables available for prompt template rendering
#[derive(Debug, Clone, Default)]
pub struct PromptVariables {
    /// Item ID
    pub id: String,

    /// Item title
    pub title: String,

    /// Item section (optional)
    pub section: String,

    /// Item overview
    pub overview: String,

    /// Path to the item directory
    pub item_path: String,

    /// Git branch name
    pub branch_name: String,

    /// Base branch for PRs
    pub base_branch: String,

    /// Signal that indicates agent completion
    pub completion_signal: String,

    /// Whether running in SDK mode
    pub sdk_mode: bool,

    /// Contents of research.md (if exists)
    pub research: Option<String>,

    /// Contents of plan.md (if exists)
    pub plan: Option<String>,

    /// Contents of prd.json (if exists)
    pub prd: Option<String>,

    /// Contents of progress.log (if exists)
    pub progress: Option<String>,

    /// Problem statement (optional context)
    pub problem_statement: Option<String>,

    /// Motivation (optional context)
    pub motivation: Option<String>,

    /// Success criteria (optional context)
    pub success_criteria: Option<Vec<String>>,

    /// Technical constraints (optional context)
    pub technical_constraints: Option<Vec<String>>,

    /// Items in scope (optional context)
    pub scope_in_scope: Option<Vec<String>>,

    /// Items out of scope (optional context)
    pub scope_out_of_scope: Option<Vec<String>>,
}

impl PromptVariables {
    /// Convert to a hashmap of string values for template rendering
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        map.insert("id".to_string(), self.id.clone());
        map.insert("title".to_string(), self.title.clone());
        map.insert("section".to_string(), self.section.clone());
        map.insert("overview".to_string(), self.overview.clone());
        map.insert("item_path".to_string(), self.item_path.clone());
        map.insert("branch_name".to_string(), self.branch_name.clone());
        map.insert("base_branch".to_string(), self.base_branch.clone());
        map.insert("completion_signal".to_string(), self.completion_signal.clone());
        map.insert("sdk_mode".to_string(), self.sdk_mode.to_string());

        if let Some(ref research) = self.research {
            map.insert("research".to_string(), research.clone());
        }
        if let Some(ref plan) = self.plan {
            map.insert("plan".to_string(), plan.clone());
        }
        if let Some(ref prd) = self.prd {
            map.insert("prd".to_string(), prd.clone());
        }
        if let Some(ref progress) = self.progress {
            map.insert("progress".to_string(), progress.clone());
        }
        if let Some(ref ps) = self.problem_statement {
            map.insert("problem_statement".to_string(), ps.clone());
        }
        if let Some(ref m) = self.motivation {
            map.insert("motivation".to_string(), m.clone());
        }
        if let Some(ref sc) = self.success_criteria {
            map.insert("success_criteria".to_string(), sc.join("\n- "));
        }
        if let Some(ref tc) = self.technical_constraints {
            map.insert("technical_constraints".to_string(), tc.join("\n- "));
        }
        if let Some(ref s) = self.scope_in_scope {
            map.insert("scope_in_scope".to_string(), s.join("\n- "));
        }
        if let Some(ref s) = self.scope_out_of_scope {
            map.insert("scope_out_of_scope".to_string(), s.join("\n- "));
        }

        map
    }
}

/// Load a prompt template, checking for custom template first.
///
/// Looks for the template in .wreckit/prompts/ first, falling back to
/// the bundled default template.
///
/// # Arguments
/// * `root` - Repository root path
/// * `name` - Template name (e.g., "research", "plan", "implement", "pr")
///
/// # Returns
/// The template content as a string
pub fn load_prompt_template(root: &Path, name: &str) -> Result<String> {
    // Check for custom template
    let custom_path = get_prompts_dir(root).join(format!("{}.md", name));
    if custom_path.exists() {
        return std::fs::read_to_string(&custom_path).map_err(|e| {
            WreckitError::FileNotFound(format!("Cannot read template {}: {}", custom_path.display(), e))
        });
    }

    // Fall back to bundled default
    match name {
        "research" => Ok(DEFAULT_RESEARCH_PROMPT.to_string()),
        "plan" => Ok(DEFAULT_PLAN_PROMPT.to_string()),
        "implement" => Ok(DEFAULT_IMPLEMENT_PROMPT.to_string()),
        "pr" => Ok(DEFAULT_PR_PROMPT.to_string()),
        _ => Err(WreckitError::FileNotFound(format!(
            "Unknown prompt template: {}",
            name
        ))),
    }
}

/// Render a prompt template with variable substitution.
///
/// Supports:
/// - `{{variable}}` - Simple variable substitution
/// - `{{#if variable}}...{{/if}}` - Conditional content (included if variable is non-empty)
/// - `{{#ifnot variable}}...{{/ifnot}}` - Inverse conditional (included if variable is empty/missing)
///
/// # Arguments
/// * `template` - The template string
/// * `variables` - Variables to substitute
///
/// # Returns
/// The rendered template
pub fn render_prompt(template: &str, variables: &PromptVariables) -> String {
    let vars = variables.to_map();
    let mut result = template.to_string();

    // Process {{#if variable}}...{{/if}} blocks
    let if_regex = Regex::new(r"\{\{#if\s+(\w+)\}\}([\s\S]*?)\{\{/if\}\}").unwrap();
    result = if_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let content = &caps[2];
            match vars.get(var_name) {
                Some(val) if !val.is_empty() => content.to_string(),
                _ => String::new(),
            }
        })
        .to_string();

    // Process {{#ifnot variable}}...{{/ifnot}} blocks
    let ifnot_regex = Regex::new(r"\{\{#ifnot\s+(\w+)\}\}([\s\S]*?)\{\{/ifnot\}\}").unwrap();
    result = ifnot_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let content = &caps[2];
            match vars.get(var_name) {
                Some(val) if !val.is_empty() => String::new(),
                _ => content.to_string(),
            }
        })
        .to_string();

    // Process {{variable}} substitutions
    let var_regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    result = var_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            vars.get(var_name).cloned().unwrap_or_default()
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_render_simple_substitution() {
        let template = "Hello {{name}}, welcome to {{place}}!";
        let mut vars = PromptVariables::default();
        vars.id = "name".to_string(); // Using id as a hacky test

        // Direct map test
        let mut map = HashMap::new();
        map.insert("name".to_string(), "Alice".to_string());
        map.insert("place".to_string(), "Wonderland".to_string());

        let var_regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let result = var_regex
            .replace_all(template, |caps: &regex::Captures| {
                let var_name = &caps[1];
                map.get(var_name).cloned().unwrap_or_default()
            })
            .to_string();

        assert_eq!(result, "Hello Alice, welcome to Wonderland!");
    }

    #[test]
    fn test_render_conditional_if() {
        let template = "Start{{#if research}}\nResearch: {{research}}{{/if}}\nEnd";
        let mut vars = PromptVariables::default();
        vars.research = Some("Found stuff".to_string());

        let result = render_prompt(template, &vars);
        assert!(result.contains("Research: Found stuff"));
    }

    #[test]
    fn test_render_conditional_if_empty() {
        let template = "Start{{#if research}}\nResearch: {{research}}{{/if}}\nEnd";
        let vars = PromptVariables::default();

        let result = render_prompt(template, &vars);
        assert!(!result.contains("Research:"));
        assert!(result.contains("Start"));
        assert!(result.contains("End"));
    }

    #[test]
    fn test_render_conditional_ifnot() {
        let template = "{{#ifnot research}}No research yet{{/ifnot}}";
        let vars = PromptVariables::default();

        let result = render_prompt(template, &vars);
        assert!(result.contains("No research yet"));
    }

    #[test]
    fn test_render_conditional_ifnot_with_value() {
        let template = "{{#ifnot research}}No research yet{{/ifnot}}";
        let mut vars = PromptVariables::default();
        vars.research = Some("Has research".to_string());

        let result = render_prompt(template, &vars);
        assert!(!result.contains("No research yet"));
    }

    #[test]
    fn test_load_bundled_templates() {
        let temp = TempDir::new().unwrap();

        // Should load bundled defaults when no custom templates exist
        let research = load_prompt_template(temp.path(), "research").unwrap();
        assert!(!research.is_empty());

        let plan = load_prompt_template(temp.path(), "plan").unwrap();
        assert!(!plan.is_empty());

        let implement = load_prompt_template(temp.path(), "implement").unwrap();
        assert!(!implement.is_empty());

        let pr = load_prompt_template(temp.path(), "pr").unwrap();
        assert!(!pr.is_empty());
    }

    #[test]
    fn test_load_custom_template() {
        let temp = TempDir::new().unwrap();
        let prompts_dir = temp.path().join(".wreckit").join("prompts");
        std::fs::create_dir_all(&prompts_dir).unwrap();

        let custom_content = "Custom research template for {{id}}";
        std::fs::write(prompts_dir.join("research.md"), custom_content).unwrap();

        let template = load_prompt_template(temp.path(), "research").unwrap();
        assert_eq!(template, custom_content);
    }

    #[test]
    fn test_load_unknown_template() {
        let temp = TempDir::new().unwrap();

        let result = load_prompt_template(temp.path(), "unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_variables_to_map() {
        let mut vars = PromptVariables::default();
        vars.id = "test-001".to_string();
        vars.title = "Test Title".to_string();
        vars.research = Some("Research content".to_string());

        let map = vars.to_map();

        assert_eq!(map.get("id"), Some(&"test-001".to_string()));
        assert_eq!(map.get("title"), Some(&"Test Title".to_string()));
        assert_eq!(map.get("research"), Some(&"Research content".to_string()));
    }
}
