# %%
import io
import re
import sqlite3
import numpy as np
from vertexai.language_models import TextEmbeddingModel

# %% Read markdown file located at /home/blmarket/note.md
contents = None
with open('/home/blmarket/note.md', 'r') as f:
  contents = f.read()

#%%
def text_embedding(text) -> list:
    """Text embedding with a Large Language Model."""
    print("CALCULATING")
    model = TextEmbeddingModel.from_pretrained("textembedding-gecko@001")
    embeddings = model.get_embeddings([text])
    for embedding in embeddings:
        vector = embedding.values
    return vector

# %%
conn = sqlite3.connect("database.db")
cursor = conn.cursor()
cursor.execute("CREATE TABLE IF NOT EXISTS embeddings (id INTEGER PRIMARY KEY, text TEXT, embedding BINARY);")
cursor.close()
conn.close()

#%%
def find_from_database(text):
   conn = sqlite3.connect("database.db")
   cursor = conn.cursor()
   cursor.execute("SELECT embedding FROM embeddings WHERE text=?;", (text,))
   row = cursor.fetchone()
   if row is None:
      return None
   bindata = io.BytesIO(row[0])
   cursor.close()
   conn.close()
   return np.load(bindata)
 
#%%
def put_into_database(text, embedding):
  buf = io.BytesIO()
  np.save(buf, embedding)
  buf.flush()
  buf.seek(0)
  bindata = buf.read()
  conn = sqlite3.connect("database.db")
  cursor = conn.cursor()
  cursor.execute("INSERT INTO embeddings (text, embedding) VALUES (?, ?);", (text, bindata))
  conn.commit()
 
#%%
def get_or_calc(text):
  entry = find_from_database(text)
  if entry is None:
    entry = text_embedding(text)
    put_into_database(text, entry)
  return entry

#%%
len(get_or_calc("test"))

# %%
# Find all markdown headers.
sections = re.finditer(r'^# (.+)$', contents, re.MULTILINE)
prev = 0

for idx, it in enumerate(sections):
  cur = it.span()[0]
  if cur == prev:
    prev = cur
    continue
  slice = contents[prev:cur]
  print(idx, slice.split("\n")[0][:64])
  emb = get_or_calc(slice)
  prev = cur
  
#%%
import sqlite3
import faiss

# Connect to the database
conn = sqlite3.connect("database.db")
cursor = conn.cursor()

# Get all the embeddings from the database
embeddings = []
texts = []
cursor.execute("SELECT text, embedding FROM embeddings")
for row in cursor:
  texts.append(row[0])
  embeddings.append(np.load(io.BytesIO(row[1])))
embeddings = np.array(embeddings)

cursor.close()
conn.close()
  
#%%
# Create a Faiss index
vector_dimension = len(embeddings[0])
index = faiss.IndexFlatL2(vector_dimension)

# Normalize the embeddings
# faiss.normalize_L2(embeddings)

# Add the embeddings to the index
index.add(embeddings)

# Close the connection to the database
# %%
target = embeddings[0]

#%%
# Find the closest embeddings
distances, indices = index.search(embeddings, 10)

#%%

# Print the closest embeddings
for i, [distance, index] in enumerate(zip(distances, indices)):
  print("-------------------")
  print(texts[i], texts[index[1]])
  # print(f"Distance: {distance}, Embedding: {embeddings[index]}")

# %%
