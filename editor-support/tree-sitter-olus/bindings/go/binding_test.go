package tree_sitter_olus_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_olus "github.com/recmo/olus/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_olus.Language())
	if language == nil {
		t.Errorf("Error loading Olus grammar")
	}
}
