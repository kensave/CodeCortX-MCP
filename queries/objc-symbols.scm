; Objective-C Symbol Extraction Queries

; Functions (C-style)
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @function.name)) @function.definition

; Classes
(class_interface
  (identifier) @class.name) @class.definition

(class_implementation
  (identifier) @class.name) @class.definition

; Protocols
(protocol_declaration
  (identifier) @interface.name) @interface.definition

; Methods
(method_definition
  (identifier) @method.name) @method.definition

(method_declaration
  (identifier) @method.name) @method.definition

; Enums
(enum_specifier
  name: (type_identifier) @enum.name) @enum.definition

; Structs
(struct_specifier
  name: (type_identifier) @struct.name) @struct.definition

; Constants
(enumerator
  name: (identifier) @constant.name) @constant.definition

; Variables
(identifier) @variable.name

; Module imports
(module_import
  (identifier) @module.name) @module.definition
