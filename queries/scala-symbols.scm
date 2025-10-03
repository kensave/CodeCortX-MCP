; Classes
(class_definition
  name: (identifier) @class.name) @class.definition

; Objects
(object_definition
  name: (identifier) @class.name) @class.definition

; Traits
(trait_definition
  name: (identifier) @interface.name) @interface.definition

; Enums
(enum_definition
  name: (identifier) @enum.name) @enum.definition

; Functions
(function_definition
  name: (identifier) @function.name) @function.definition

(function_declaration
  name: (identifier) @function.name) @function.definition

; Values/Variables
(val_definition
  pattern: (identifier) @variable.name) @variable.definition

(var_definition
  pattern: (identifier) @variable.name) @variable.definition

; Type definitions
(type_definition
  name: (type_identifier) @class.name) @class.definition

; Package declarations
(package_clause
  name: (package_identifier) @module.name) @module.definition
