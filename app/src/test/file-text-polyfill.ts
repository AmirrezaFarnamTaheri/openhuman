/**
 * Match browser File.text() in jsdom versions which omit that standard API.
 * Read actual fixture bytes; do not stub or short-circuit certificate uploads.
 */
if (typeof File !== 'undefined' && typeof File.prototype.text !== 'function') {
  Object.defineProperty(File.prototype, 'text', {
    configurable: true,
    value(this: File): Promise<string> {
      return new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result ?? ''));
        reader.onerror = () => reject(reader.error ?? new Error('File read failed'));
        reader.onabort = () => reject(new Error('File read cancelled'));
        reader.readAsText(this);
      });
    },
  });
}
