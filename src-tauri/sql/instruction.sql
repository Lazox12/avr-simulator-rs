CREATE TABLE IF NOT EXISTS instruction(
    address INT PRIMARY KEY NOT NULL,
    opcode TEXT,
    rawOpcode INT,
    operands TEXT,
    comment TEXT,
    commentDisplay TEXT
)