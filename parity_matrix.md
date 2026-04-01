# API Parity Matrix

Track each row with one of: ⬜ Not started / 🔄 In progress / ✅ Done / ❌ Blocked

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `NewClient` | `Client::new()` | — | — | ✅ | ✅ |
| `option.WithAPIKey` | `ClientConfig::with_api_key()` | — | — | ✅ | ✅ |
| `option.WithBaseURL` | `ClientConfig::with_base_url()` | — | — | ✅ | ✅ |
| `ChatCompletions.New` | `chat().completions().create()` | ✅ | ✅ | ✅ | ✅ |
| `ChatCompletions.NewStreaming` | `chat().completions().create_stream()` | ✅ | ✅ | ✅ | ✅ |
| `Completions.New` | `completions().create()` | ✅ | ✅ | ✅ | ✅ |
| `Embeddings.New` | `embeddings().create()` | ✅ | ✅ | ✅ | ✅ |
| `Models.List` | `models().list()` | — | ✅ | ✅ | ✅ |
| `Models.Get` | `models().get()` | — | ✅ | ✅ | ✅ |
| `Models.Delete` | `models().delete()` | — | ✅ | ✅ | ✅ |
| `Moderations.New` | `moderations().create()` | ✅ | ✅ | ✅ | ✅ |
| `Files.New` | `files().create()` | ✅ | ✅ | ✅ | ✅ |
| `Files.List` | `files().list()` | — | ✅ | ✅ | ✅ |
| `Files.Get` | `files().get()` | — | ✅ | ✅ | ✅ |
| `Files.Delete` | `files().delete()` | — | ✅ | ✅ | ✅ |
| `Files.Content` | `files().content()` | — | ✅ | ✅ | ✅ |
| `Uploads.New` | `uploads().create()` | ✅ | ✅ | ✅ | ✅ |
| `Uploads.Parts.New` | `uploads().create_part()` | ✅ | ✅ | ✅ | ✅ |
| `Uploads.Complete` | `uploads().complete()` | ✅ | ✅ | ✅ | ✅ |
| `Uploads.Cancel` | `uploads().cancel()` | — | ✅ | ✅ | ✅ |
| `Uploads.Get` | `uploads().get()` | — | ✅ | ✅ | ✅ |
| `Audio.Transcriptions.New` | `audio().transcribe()` | ✅ | ✅ | ✅ | ✅ |
| `Audio.Translations.New` | `audio().translate()` | ✅ | ✅ | ✅ | ✅ |
| `Audio.Speech.New` | `audio().speech()` | ✅ | ✅ | ✅ | ✅ |
| `Images.Generations.New` | `images().generate()` | ✅ | ✅ | ✅ | ✅ |
| `Images.Edits.New` | `images().edit()` | ✅ | ✅ | ✅ | ✅ |
| `Images.Variations.New` | `images().create_variation()` | ✅ | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.New` | `fine_tuning().jobs().create()` | ✅ | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.Get` | `fine_tuning().jobs().get()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.List` | `fine_tuning().jobs().list()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.Cancel` | `fine_tuning().jobs().cancel()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.ListEvents` | `fine_tuning().jobs().list_events()` | — | ✅ | ✅ | ✅ |
| `Batches.New` | `batches().create()` | ✅ | ✅ | ✅ | ✅ |
| `Batches.Get` | `batches().get()` | — | ✅ | ✅ | ✅ |
| `Batches.List` | `batches().list()` | — | ✅ | ✅ | ✅ |
| `Batches.Cancel` | `batches().cancel()` | — | ✅ | ✅ | ✅ |
| `VectorStores.New` | `vector_stores().create()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Get` | `vector_stores().get()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Update` | `vector_stores().update()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.List` | `vector_stores().list()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Delete` | `vector_stores().delete()` | — | ✅ | ✅ | ✅ |
| `Graders.Namespace` | `graders().grader_models()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.Assistants.New` | `beta().assistants().create()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.Assistants.Get` | `beta().assistants().get()` | — | ✅ | ✅ | ✅ |
| `Beta.Assistants.Update` | `beta().assistants().update()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.Assistants.List` | `beta().assistants().list()` | — | ✅ | ✅ | ✅ |
| `Beta.Assistants.Delete` | `beta().assistants().delete()` | — | ✅ | ✅ | ✅ |
| `Beta.Threads.New` | `beta().threads().create()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.Threads.Get` | `beta().threads().get()` | — | ✅ | ✅ | ✅ |
| `Beta.Threads.Update` | `beta().threads().update()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.Threads.Delete` | `beta().threads().delete()` | — | ✅ | ✅ | ✅ |
| `Beta.ThreadRuns.New` | `beta().threads().runs().create()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.ThreadRuns.Get` | `beta().threads().runs().get()` | — | ✅ | ✅ | ✅ |
| `Beta.ThreadRuns.List` | `beta().threads().runs().list()` | — | ✅ | ✅ | ✅ |
| `Beta.ThreadRuns.Cancel` | `beta().threads().runs().cancel()` | — | ✅ | ✅ | ✅ |
