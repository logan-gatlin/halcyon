use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleStatementKind {
    DocComment(String),
    Let {
        assignee: PatternExpression,
        value: Box<ValueExpression>,
    },
    Type {
        assignee: Spanned<String>,
        value: Box<TypeDefinition>,
    },
}

pub type ModuleStatement = Expression<ModuleStatementKind>;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerParsedModule {
    pub name: Spanned<String>,
    pub contents: Vec<ModuleStatement>,
}

pub type ParsedModule = Spanned<InnerParsedModule>;

const RECOVERY: RecoveryBehavior = UntilNextStatement;

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
    pub fn parse_module_statement(&mut self) -> Result<ModuleStatement> {
        use TokenKind::*;
        let next = self.next_token_or_err().ok_or(NoRecovery)?;
        let span = next.span;
        Ok(match next.inner {
            Module => {
                self.error()
                    .primary("Expected `end` here.", span)
                    .note("Nested modules are not allowed.")
                    .done();
                return Err(RECOVERY);
            }
            End => {
                unreachable!(
                    "The `end` token should be handled outside of `parse_module_statement`, because it marks the end of a module"
                );
            }
            DocComment(comment) => {
                ModuleStatementKind::DocComment(comment)
            }
            Let => {
                let assignee = self.parse_pattern()?;
                self.eat_or_err(&TokenKind::Equal).ok_or(RECOVERY)?;
                let value = self.parse_value_expression(0)?.into();
                ModuleStatementKind::Let {
                    assignee,
                    value,
                }
            }
            Type => {
                let assignee = self.eat_ident_or_err().ok_or(RECOVERY)?;
                self.eat_or_err(&TokenKind::Equal).ok_or(RECOVERY)?;
                let value = self.parse_type_definition()?.into();
                ModuleStatementKind::Type {
                    assignee,
                    value,
                }
            }
            _ => {
                self.error().primary("Expected a `let` or `type` statement here", span).done();
                return Err(RECOVERY);
            },
        }.with_span(span + self.last_span))
    }
    pub fn parse_module(&mut self) -> Option<ParsedModule> {
        if self.eat_or_err(&Module).is_none() {
            self.recover(UntilKind(TokenKind::Module));
            return None;
        };
        let span = self.last_span;
        let name = match self.eat_ident_or_err() {
            Some(name) => name,
            None => {
                self.recover(UntilKind(TokenKind::End));
                return None;
            }
        };
        let _ = self.eat_or_err(&Equal);
        let mut contents = vec![];
        while self.peek().is_some_and(|t| t.inner != End) {
            match self.parse_module_statement() {
                Ok(s) => contents.push(s),
                Err(recovery) => self.recover(recovery),
            };
        }
        let _ = self.eat_or_err(&End);
        Some(InnerParsedModule { name, contents }.with_span(span + self.last_span))
    }
}
