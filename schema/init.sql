CREATE TABLE users(
    id SERIAL NOT NULL,
    email text NOT NULL,
    password text NOT NULL,
    name text NULL,
    address text NULL,
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

CREATE TYPE level_alert AS ENUM ('One', 'Two', 'Three');

CREATE TABLE alerts(
    id SERIAL NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    area GEOMETRY(Polygon, 4326),
    level level_alert NOT NULL,
    reached_users INTEGER DEFAULT 0 NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk_users_id
        FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE notifications(
    id SERIAL NOT NULL,
    alert_id INTEGER NOT NULL,
    position_id INTEGER NOT NULL,
    seen BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(id),
    CONSTRAINT fk_alerts_id
        FOREIGN KEY(alert_id) REFERENCES alerts(id),
    CONSTRAINT fk_positions_id
        FOREIGN KEY(position_id) REFERENCES positions(id)
);
