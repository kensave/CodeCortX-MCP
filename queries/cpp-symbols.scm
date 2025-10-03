; C++ Symbol Extraction Queries

; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @function.name)) @function.definition

(function_definition
  declarator: (function_declarator
    declarator: (qualified_identifier
      name: (identifier) @function.name))) @function.definition

; Classes
(class_specifier
  name: (type_identifier) @class.name) @class.definition

(struct_specifier
  name: (type_identifier) @class.name) @class.definition

; Namespaces
(namespace_definition
  name: (namespace_identifier) @module.name) @module.definition

; Enums
(enum_specifier
  name: (type_identifier) @enum.name) @enum.definition

; Variables/Constants
(declaration
  declarator: (init_declarator
    declarator: (identifier) @variable.name)) @variable.definition

(declaration
  declarator: (identifier) @variable.name) @variable.definition

; Type aliases
(alias_declaration
  name: (type_identifier) @type.name) @type.definition

; Templates
(template_declaration
  (class_specifier
    name: (type_identifier) @class.name)) @class.definition

(template_declaration
  (function_definition
    declarator: (function_declarator
      declarator: (identifier) @function.name))) @function.definition



; Operator overloads
(function_definition
  declarator: (function_declarator
    declarator: (operator_name) @function.name)) @function.definition
