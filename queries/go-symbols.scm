; Functions
(function_declaration
  name: (identifier) @function.name) @function.definition

; Methods
(method_declaration
  name: (field_identifier) @function.name) @function.definition

; Type declarations (structs, interfaces, etc.)
(type_spec
  name: (type_identifier) @class.name) @class.definition

; Type aliases
(type_alias
  name: (type_identifier) @class.name) @class.definition

; Constants
(const_spec
  name: (identifier) @variable.name) @variable.definition

; Variables
(var_spec
  name: (identifier) @variable.name) @variable.definition

; Short variable declarations
(short_var_declaration
  left: (expression_list
    (identifier) @variable.name)) @variable.definition

; Package declarations
(package_clause
  (package_identifier) @module.name) @module.definition

; Import declarations
(import_spec
  path: (interpreted_string_literal) @import.name) @import.definition
