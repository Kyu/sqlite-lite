pub(crate) enum StatementType {
    Invalid,
    Insert,
    Select
}

pub(crate) enum MetaCommandResult {
    MetaCommandSuccess,
    MetaCommandUnrecognized
}

pub(crate) enum PreparedStatementResult {
    PreparedStatementSuccess,
    PreparedStatementSyntaxError,
    PreparedStatementUnrecognized,
    PreparedStatementError
}

pub(crate) enum ExecuteResult {
    ExecuteSuccess,
    ExecuteFailed,
    ExecuteTableFull
}