; Classes
(class_declaration
  name: (identifier) @class.name) @class.definition

; Interfaces
(interface_declaration
  name: (identifier) @interface.name) @interface.definition

; Enums
(enum_declaration
  name: (identifier) @enum.name) @enum.definition

; Records
(record_declaration
  name: (identifier) @class.name) @class.definition

; Methods
(method_declaration
  name: (identifier) @function.name) @function.definition

; Constructors
(constructor_declaration
  name: (identifier) @function.name) @function.definition

; Fields/Variables
(field_declaration
  declarator: (variable_declarator
    name: (identifier) @variable.name)) @variable.definition

; Local variables
(local_variable_declaration
  declarator: (variable_declarator
    name: (identifier) @variable.name)) @variable.definition

; Enum constants
(enum_constant
  name: (identifier) @variable.name) @variable.definition

; Package declarations
(package_declaration
  (identifier) @module.name) @module.definition

; Import statements
(import_declaration
  (identifier) @import.name) @import.definition
