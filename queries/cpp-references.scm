; C++ Reference Tracking Queries

; Function calls
(call_expression
  function: (identifier) @reference.name) @reference.usage

(call_expression
  function: (qualified_identifier
    name: (identifier) @reference.name)) @reference.usage

(call_expression
  function: (field_expression
    field: (field_identifier) @reference.name)) @reference.usage

; Variable references
(identifier) @reference.name

; Field access
(field_expression
  field: (field_identifier) @reference.name) @reference.usage

; Type references
(type_identifier) @reference.name

; Qualified identifiers
(qualified_identifier
  name: (identifier) @reference.name) @reference.usage

; Template instantiations
(template_type
  name: (type_identifier) @reference.name) @reference.usage

(template_function
  name: (identifier) @reference.name) @reference.usage

; Namespace usage
(qualified_identifier
  scope: (namespace_identifier) @reference.name) @reference.usage
