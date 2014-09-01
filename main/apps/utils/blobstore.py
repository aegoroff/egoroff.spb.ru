from exceptions import KeyError, ValueError
from google.appengine.ext import blobstore
from werkzeug.wrappers import Response
from apps.utils import blobstore_handlers


def get_uploads(request, field_name=None):
    __uploads = {}
    for key, value in request.files.items():
#        if isinstance(value, FileStorage):
        for option in value.headers['Content-Type'].split(';'):
            if 'blob-key' in option:
                __uploads.setdefault(key, []).append(
                    blobstore_handlers.parse_blob_info(value))

    if field_name:
        try:
            return list(__uploads[field_name])
        except KeyError:
            return []
    else:
        results = []
        for uploads in __uploads.itervalues():
            results += uploads

    return results

def send_blob(blob_key_or_info, content_type=None, save_as=None):
   if isinstance(blob_key_or_info, blobstore.BlobInfo):
     blob_key = blob_key_or_info.key()
     blob_info = blob_key_or_info
   elif isinstance(blob_key_or_info, str) and blob_key_or_info.startswith(
       '/gs/'):
     blob_key = blobstore.create_gs_key(blob_key_or_info)
     blob_info = None
   else:
     blob_key = blob_key_or_info
     blob_info = None

   headers = {blobstore.BLOB_KEY_HEADER: str(blob_key)}

   if content_type:
     if isinstance(content_type, unicode):
       content_type = content_type.encode('utf-8')
     headers['Content-Type'] = content_type
   else:
     headers['Content-Type'] = ''

   def send_attachment(filename):
     if isinstance(filename, unicode):
       filename = filename.encode('utf-8')
     headers['Content-Disposition'] = (
         blobstore_handlers._CONTENT_DISPOSITION_FORMAT % filename)

   if save_as:
     if isinstance(save_as, basestring):
       send_attachment(save_as)
     elif blob_info and save_as is True:
       send_attachment(blob_info.filename)
     else:
       if not blob_info:
         raise ValueError('Expected BlobInfo value for blob_key_or_info.')
       else:
         raise ValueError('Unexpected value for save_as.')

   return Response('', headers=headers)
