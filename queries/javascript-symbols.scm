; Classes
(class_declaration
  name: (identifier) @class.name) @class.definition

; Functions
(function_declaration
  name: (identifier) @function.name) @function.definition

; Methods
(method_definition
  name: (property_identifier) @method.name) @method.definition

; Variables and constants
(variable_declarator
  name: (identifier) @variable.name) @variable.definition

; Arrow functions assigned to variables
(variable_declarator
  name: (identifier) @function.name
  value: (arrow_function)) @function.definition
