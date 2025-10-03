; Classes
(class_declaration
  name: (identifier) @class.name) @class.definition

; Interfaces  
(interface_declaration
  name: (identifier) @interface.name) @interface.definition

; Methods
(method_declaration
  name: (identifier) @method.name) @method.definition

; Properties
(property_declaration
  name: (identifier) @property.name) @property.definition

; Fields
(field_declaration
  (variable_declaration
    (variable_declarator
      name: (identifier) @field.name))) @field.definition

; Enums
(enum_declaration
  name: (identifier) @enum.name) @enum.definition

; Structs
(struct_declaration
  name: (identifier) @struct.name) @struct.definition

; Delegates
(delegate_declaration
  name: (identifier) @delegate.name) @delegate.definition

; Constructors
(constructor_declaration
  name: (identifier) @constructor.name) @constructor.definition

; Records
(record_declaration
  name: (identifier) @record.name) @record.definition

; Namespaces 
(namespace_declaration
  name: (_) @namespace.name) @namespace.definition
