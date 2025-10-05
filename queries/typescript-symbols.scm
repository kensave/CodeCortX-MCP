; TypeScript/TSX Symbol Extraction Queries

; Functions
(function_declaration
  name: (identifier) @function.name) @function.definition

(function_expression
  name: (identifier) @function.name) @function.definition

(arrow_function
  parameter: (identifier) @function.name) @function.definition

(method_definition
  name: (_) @method.name) @method.definition

; Classes
(class_declaration
  name: (type_identifier) @class.name) @class.definition

(class
  name: (type_identifier) @class.name) @class.definition

(abstract_class_declaration
  name: (type_identifier) @class.name) @class.definition

; Interfaces
(interface_declaration
  name: (type_identifier) @interface.name) @interface.definition

; Types
(type_alias_declaration
  name: (type_identifier) @type.name) @type.definition

; Enums
(enum_declaration
  name: (identifier) @enum.name) @enum.definition

; Variables/Constants
(variable_declarator
  name: (identifier) @variable.name) @variable.definition

(lexical_declaration
  (variable_declarator
    name: (identifier) @variable.name)) @variable.definition

; Modules/Namespaces
(module
  name: (_) @module.name) @module.definition

(internal_module
  name: (_) @module.name) @module.definition

; Imports
(import_statement
  (import_clause
    (identifier) @import.name)) @import.definition

(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (_) @import.name)))) @import.definition

; Exports
(export_statement
  (export_clause
    (export_specifier
      name: (_) @export.name))) @export.definition
