CREATE TABLE users(
    id SERIAL NOT NULL,
    email text NOT NULL,
    password text NOT NULL,
    is_admin boolean default false,
    PRIMARY KEY (id)
);

CREATE TYPE moving_activity AS ENUM ('InVehicle', 'OnFoot', 'Running', 'Walking', 'Still');

CREATE TABLE positions(
    id SERIAL NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    location GEOGRAPHY(Point, 4326) NOT NULL,
    activity moving_activity NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk_users_id
        FOREIGN KEY(user_id) REFERENCES users(id)
);
