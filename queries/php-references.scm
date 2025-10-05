; PHP Reference Extraction Queries

; Function calls
(function_call_expression
  function: (name) @reference.name) @reference.usage

; Method calls
(member_call_expression
  name: (name) @reference.name) @reference.usage

; Static method calls
(scoped_call_expression
  name: (name) @reference.name) @reference.usage

; Class instantiation
(object_creation_expression
  (name) @reference.name) @reference.usage

; Variable references
(variable_name) @reference.name

; Class constant access
(class_constant_access_expression
  (name) @reference.name) @reference.usage

; Property access
(member_access_expression
  name: (name) @reference.name) @reference.usage

; Static property access
(scoped_property_access_expression
  name: (variable_name) @reference.name) @reference.usage

; Namespace usage
(namespace_use_declaration
  (namespace_use_clause
    (name) @reference.name)) @reference.usage
