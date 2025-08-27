import { useEffect, useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { listDocuments, addDocument } from "@/lib/api";

export default function DocumentsPage() {
  const [file, setFile] = useState<File | null>(null);
  const [docs, setDocs] = useState<{ id: number; filename: string; size: number; uploaded_at: number }[]>([]);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    void (async () => {
      const list = await listDocuments();
      setDocs(list);
    })();
  }, []);

  const onUpload = async () => {
    if (!file) return;
    setUploading(true);
    // read file bytes (not necessary but realistic)
    try {
      const buf = await file.arrayBuffer();
      await addDocument({ filename: file.name, size: file.size, data: new Uint8Array(buf) });
      const list = await listDocuments();
      setDocs(list);
      setFile(null);
    } finally {
      setUploading(false);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-semibold mb-4">Documents</h1>
      <div className="grid gap-6 md:grid-cols-3">
        <Card className="shadow-card md:col-span-2 p-4">
          <CardContent>
            <div className="space-y-4">
              <Input type="file" onChange={(e) => setFile(e.target.files?.[0] ?? null)} />
              <Button disabled={!file || uploading} onClick={() => void onUpload()}>{uploading ? 'Uploading...' : 'Upload (simulate)'}</Button>
              <div>
                <h3 className="font-medium">Uploaded documents</h3>
                <ul className="mt-2 space-y-2">
                  {docs.map(d => (
                    <li key={d.id} className="p-2 border rounded flex justify-between items-center">
                      <div>
                        <div className="font-medium">{d.filename}</div>
                        <div className="text-sm text-muted-foreground">{Math.round(d.size / 1024)} KB • {new Date(d.uploaded_at).toLocaleString()}</div>
                      </div>
                      <div>
                        <a className="text-blue-600" href="#">Download</a>
                      </div>
                    </li>
                  ))}
                  {docs.length === 0 && <li className="text-sm text-muted-foreground">No documents uploaded yet.</li>}
                </ul>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card className="shadow-card">
          <CardContent>
            <h3 className="text-sm font-medium mb-2">Upload Tips</h3>
            <div className="space-y-2 text-sm text-muted-foreground">
              <div>Large files are chunked and processed in the background (simulated).</div>
              <div>Encrypted documents will be available after processing.</div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
