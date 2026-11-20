import { firmwareUploadPath } from "./protocol";
import { RET } from "./types";

interface uploadCallback {
  progress: (progress: number) => void,
  status: (status: RET) => void
}

/**
 * Upload firmware to wnb
 * @param  {string}                 authToken AuthToken
 * @param  {XMLHttpRequestBodyInit} file      File to upload
 * @return {uploadCallback}                   uploadCallback
 */
export function firmwareUpgrade(authToken: string, file: XMLHttpRequestBodyInit): uploadCallback {
  const xhr = new XMLHttpRequest();
  var ret = {} as uploadCallback;
  xhr.open('POST', firmwareUploadPath);
  xhr.onreadystatechange = () => {
    if (xhr.readyState !== 4) return;
    // 401 = token expired; anything else but 200 = image rejected (size, header, flash write)
    ret.status(xhr.status === 200 ? RET.OK : xhr.status === 401 ? RET.AUTH_FAIL : RET.FAIL);
  }
  xhr.upload.onerror = () => {
    ret.progress(0);
    ret.status(RET.FAIL);
  }
  xhr.upload.onprogress = (event) => {
    if (event.lengthComputable) {
      ret.progress(Math.round(event.loaded / event.total * 100));
    }
  }
  xhr.setRequestHeader('Content-Type', 'data/binary');
  xhr.setRequestHeader('Auth-Token', authToken);
  xhr.responseType = 'json';
  xhr.send(file);
  return ret;
}
