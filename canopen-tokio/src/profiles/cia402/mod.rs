pub enum State {
    ///cannot be switched to deliberately
    NotReadyToSwitchOn,
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    /// cannot be switched to deliberately
    Fault,
    /// cannot be switched to deliberately
    FaultReactionActive,
    QuickStopActive,
    /// only as a command when writing
    DisableVoltage,
}
