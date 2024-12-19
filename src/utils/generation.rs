use chrono::{Datelike, Utc};
use serde::Serialize;
use std::{collections::HashMap, env};
use tera::{Context, Tera};

use crate::utils::{
    helpers::sanitize_name,
    variables::{SUI_PROJECT, SUI_PROJECT_SUB_DIR},
};

// Struct representing the package details in the Move.toml file
#[derive(Serialize)]
struct Package {
    name: String,    // The name of the package
    edition: String, // The edition of the package
    version: String, // The version of the package
}

// Struct representing the dependencies of the Move package, specifically the SUI dependency
#[derive(Serialize)]
struct Dependency {
    #[serde(rename = "Sui")] // Rename to "Sui" in the serialized output
    sui: SuiDependency, // The SUI dependency details
}

// Struct representing the SUI dependency details
#[derive(Serialize)]
struct SuiDependency {
    git: String,    // Git URL for the SUI repository
    subdir: String, // Subdirectory path within the repository
    rev: String,    // The revision or branch of the SUI repository
}

// Struct representing the entire content of the Move.toml file, including package, dependencies, and addresses
#[derive(Serialize)]
struct MoveToml {
    package: Package,                   // The package details
    dependencies: Dependency,           // The dependencies of the package
    addresses: HashMap<String, String>, // Mapping of package names to their respective addresses
}

// Function to generate token content based on user inputs, such as token decimals, symbol, name, and description
// It uses a templating system (Tera) to generate the contract code from templates.
pub fn generate_token(
    decimals: u8,        // The number of decimals for the token
    symbol: String,      // The symbol of the token
    name: String,        // The name of the token
    description: String, // The description of the token
    is_frozen: bool,     // Flag to indicate if the token is frozen
    is_test: bool,       // Flag to indicate if the generated token is for testing
) -> String {
    // Sanitize the token name to be alphanumeric
    let slug = sanitize_name(&name.to_string());

    let module_name = slug.clone(); // Module name based on sanitized name
    let token_type = slug.to_uppercase(); // Token type (uppercased sanitized name)

    // Get the current working directory
    let current_dir = env::current_dir().unwrap();
    // Define the path to the template files
    let templates_path = format!("{}/src/templates/**/*", current_dir.display());

    // Initialize the Tera templating engine with the specified template path
    let tera = Tera::new(&templates_path).unwrap();

    // Create a new Tera context to insert variables into the template
    let mut context = Context::new();
    context.insert("module_name", &module_name); // Insert the module name
    context.insert("token_type", &token_type); // Insert the token type
    context.insert("name", &name); // Insert the token name
    context.insert("symbol", &symbol); // Insert the token symbol
    context.insert("decimals", &decimals); // Insert the token decimals
    context.insert("description", &description); // Insert the token description
    context.insert("is_frozen", &is_frozen); // Insert the token freeze status

    // Select the appropriate template file based on whether it's for testing or not
    let template_file = if is_test {
        "test_token_template.move" // Use the test token template for testing
    } else {
        "token_template.move" // Use the regular token template
    };

    // Render the token template with the provided context (variables)
    let token_template: String = tera.render(template_file, &context).unwrap();

    token_template // Return the generated token contract content
}

// Function to generate the Move.toml file with basic requirements
// It uses the provided package name and environment to generate the TOML content
pub fn generate_move_toml(package_name: String, environment: String) -> String {
    // Get the current year
    let current_year: u32 = Utc::now().year_ce().1;

    // Create the Move.toml content using the provided package name and environment
    let move_toml = MoveToml {
        package: Package {
            name: package_name.to_lowercase(), // Package name in lowercase
            edition: format!("{}.beta", current_year), // Package edition (current year + "beta")
            version: "0.0.1".to_string(),      // Package version
        },
        dependencies: Dependency {
            sui: SuiDependency {
                git: SUI_PROJECT.to_string(),              // Git URL for the SUI project
                subdir: SUI_PROJECT_SUB_DIR.to_string(),   // Subdirectory within the SUI project
                rev: format!("framework/{}", environment), // SUI project revision based on environment
            },
        },
        addresses: {
            // Mapping package name to an address (0x0 as a placeholder)
            let mut addresses = HashMap::new();
            addresses.insert(package_name.to_string(), "0x0".to_string());
            addresses
        },
    };

    // Serialize the MoveToml structure into a TOML string
    let toml_content: String = toml::to_string(&move_toml).unwrap();

    toml_content // Return the generated TOML content
}
