; Functions
(function_item 
  name: (identifier) @function.name) @function.definition

; Structs
(struct_item 
  name: (type_identifier) @struct.name) @struct.definition

; Enums
(enum_item
  name: (type_identifier) @enum.name) @enum.definition

; Unions
(union_item
  name: (type_identifier) @class.name) @class.definition

; Traits
(trait_item
  name: (type_identifier) @interface.name) @interface.definition

; Type aliases
(type_item
  name: (type_identifier) @class.name) @class.definition

; Constants
(const_item
  name: (identifier) @const.name) @const.definition

; Static items
(static_item
  name: (identifier) @static.name) @static.definition

; Modules
(mod_item
  name: (identifier) @module.name) @module.definition

; Use declarations
(use_declaration
  argument: (identifier) @import.name) @import.definition
