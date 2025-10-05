; Classes
(class_declaration
  name: (identifier) @class.name) @class.definition

; Functions
(function_declaration
  name: (identifier) @function.name) @function.definition

; Properties/Variables
(property_declaration
  (identifier) @variable.name) @variable.definition

; Objects
(object_declaration
  name: (identifier) @class.name) @class.definition

; Type aliases
(type_alias
  (identifier) @class.name) @class.definition
