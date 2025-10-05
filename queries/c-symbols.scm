; C Symbol Extraction Queries

; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @function.name)) @function.definition

(function_definition
  declarator: (function_declarator
    declarator: (pointer_declarator
      declarator: (identifier) @function.name))) @function.definition

; Structs
(struct_specifier
  name: (type_identifier) @struct.name) @struct.definition

; Unions
(union_specifier
  name: (type_identifier) @struct.name) @struct.definition

; Enums
(enum_specifier
  name: (type_identifier) @enum.name) @enum.definition

; Variables/Constants
(declaration
  declarator: (init_declarator
    declarator: (identifier) @variable.name)) @variable.definition

(declaration
  declarator: (identifier) @variable.name) @variable.definition

; Typedefs
(type_definition
  declarator: (type_identifier) @type.name) @type.definition

; Preprocessor defines
(preproc_def
  name: (identifier) @constant.name) @constant.definition

(preproc_function_def
  name: (identifier) @function.name) @function.definition

; Enumerators
(enumerator
  name: (identifier) @constant.name) @constant.definition
