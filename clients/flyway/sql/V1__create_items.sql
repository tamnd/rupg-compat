CREATE TYPE compat_mood AS ENUM ('sad', 'ok', 'happy');
CREATE TABLE compat_items (
    id int4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL UNIQUE,
    price numeric(10, 2),
    mood compat_mood NOT NULL DEFAULT 'ok'
);
INSERT INTO compat_items (name, price) VALUES ('one', 1.50), ('two', 2.50);
