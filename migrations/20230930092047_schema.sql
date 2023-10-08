CREATE TABLE subjects(
       id INT NOT NULL UNIQUE PRIMARY KEY,
       title TEXT NOT NULL,
       gang TEXT NOT NULL,
       optional INT NOT NULL
);

CREATE TABLE meetings(
       id INT NOT NULL UNIQUE PRIMARY KEY,
       name TEXT NOT NULL,
       gang TEXT NOT NULL,
       link TEXT NOT NULL
);

CREATE TABLE schedule(
       day INT NOT NULL,
       repeat INT NOT NULL,
       slot INT NOT NULL,
       subject_id INT NOT NULL,
       FOREIGN KEY(subject_id) REFERENCES subjects(id)
);

CREATE TABLE assigned(
       meeting_id INT NOT NULL,
       subject_id INT NOT NULL,
       FOREIGN KEY(meeting_id) REFERENCES meetings(id),
       FOREIGN KEY(subject_id) REFERENCES subjects(id)
);
