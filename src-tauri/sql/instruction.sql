CREATE TABLE IF NOT EXISTS instruction(
    address INT PRIMARY KEY NOT NULL,
    opcode INT,
    rawOpcode INT,
    operands TEXT,
    comment TEXT,
    commentDisplay TEXT
)