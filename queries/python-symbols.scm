; Functions
(function_definition
  name: (identifier) @function.name) @function.definition

; Classes
(class_definition
  name: (identifier) @class.name) @class.definition

; Variables/Constants (assignments)
(assignment
  left: (identifier) @variable.name) @variable.definition

; Import statements
(import_statement
  name: (dotted_name) @import.name) @import.definition

(import_from_statement
  module_name: (dotted_name) @import.module
  name: (dotted_name) @import.name) @import.definition
