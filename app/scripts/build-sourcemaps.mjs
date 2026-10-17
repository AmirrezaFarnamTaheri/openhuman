/**
 * Source maps can dominate the size of a shipped frontend artifact.
 * Keep them only when Sentry will upload/delete them or a developer requests
 * local diagnostic maps. The production default is a smaller dist directory.
 */
export function shouldGenerateSourcemaps(env = process.env) {
  return Boolean(env.SENTRY_AUTH_TOKEN?.trim()) ||
    env.OPENHUMAN_BUILD_SOURCEMAPS === "1";
}
