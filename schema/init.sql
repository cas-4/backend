CREATE TABLE users(
    id SERIAL NOT NULL,
    email text NOT NULL,
    password text NOT NULL,
    is_admin boolean default false,
    PRIMARY KEY (id)
);
