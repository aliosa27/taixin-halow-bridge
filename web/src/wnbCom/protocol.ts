export const settingsPath = '/post.action';
export const firmwareUploadPath = '/fwupg.data';
export const authPath = '/auth.action';

export const US = String.fromCharCode(31);
export const GS = String.fromCharCode(29);

/**
 * Wrapper for fetch API with timeout
 * @param  {RequestInfo |    URL}         input fetch's input
 * @param  {RequestInit}    init fetch's resp init option
 * @return {Promise}             null if timeout, standard fetch resp if OK
 */
export async function fetchTimeOut(input: RequestInfo | URL, init?: RequestInit): Promise<Response | null> {
  const abortController = new AbortController();
  const { signal } = abortController;
  const fetchPromise = fetch(input, { ...init, signal });
  const timer = setTimeout(() => abortController.abort(), 5000);

  try {
    const resp = await fetchPromise;
    clearTimeout(timer);
    return resp;
  } catch {
    clearTimeout(timer);
    return null;
  }
}

/**
 * WNB Fetch Options
 */
export class fetchOptions implements RequestInit {
  method: string | undefined;
  headers: HeadersInit | undefined;
  body: BodyInit | null | undefined;
  constructor(body: string | ArrayBuffer, token: string) {
    this.method = 'POST';
    this.headers = {
      'Content-Type': 'text/plain; charset=UTF-8',
      'Auth-Token': token
    };
    this.body = body;
  }
}
