CREATE TABLE IF NOT EXISTS public.users (
    id BIGSERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    pwhash TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS user_creds ON public.users (email, pwhash);

CREATE TABLE IF NOT EXISTS public.saved_gifs (
    id BIGSERIAL PRIMARY KEY,
    "user" BIGINT NOT NULL REFERENCES public.users,
    giphy_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL,
    category TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS user_gif_unique ON public.saved_gifs ("user", giphy_id);
