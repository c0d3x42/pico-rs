# lookups

lookup tables should be scoped to the rule file inclusion hierarchy.
each table is available to RuleFile that defined it, **and** files included below it
the lowest level included RuleFile has access to all lookups above it.
the root level RuleFile **onl** has access to the lookups defined in it


