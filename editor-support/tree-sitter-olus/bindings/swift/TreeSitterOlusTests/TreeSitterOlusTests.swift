import XCTest
import SwiftTreeSitter
import TreeSitterOlus

final class TreeSitterOlusTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_olus())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Olus grammar")
    }
}
