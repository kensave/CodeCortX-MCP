; PHP Symbol Extraction Queries

; Functions
(function_definition
  name: (name) @function.name) @function.definition

; Methods
(method_declaration
  name: (name) @method.name) @method.definition

; Classes
(class_declaration
  name: (name) @class.name) @class.definition

; Interfaces
(interface_declaration
  name: (name) @interface.name) @interface.definition

; Traits
(trait_declaration
  name: (name) @trait.name) @trait.definition

; Enums
(enum_declaration
  name: (name) @enum.name) @enum.definition

; Constants
(const_declaration
  (const_element
    (name) @constant.name)) @constant.definition

; Properties
(property_declaration
  (property_element
    name: (variable_name) @property.name)) @property.definition

; Namespaces
(namespace_definition
  name: (namespace_name) @namespace.name) @namespace.definition

; Variables
(assignment_expression
  left: (variable_name) @variable.name) @variable.definition
