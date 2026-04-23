# API Parity Matrix

Track each row with one of: ⬜ Not started / 🔄 In progress / ✅ Done / ❌ Blocked

## Client & Configuration

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `NewClient` | `Client::new()` | — | — | ✅ | ✅ |
| `option.WithAPIKey` | `ClientConfig::with_api_key()` | — | — | ✅ | ✅ |
| `option.WithBaseURL` | `ClientConfig::with_base_url()` | — | — | ✅ | ✅ |
| `option.WithOrganization` | `ClientConfig::with_organization()` | — | — | ✅ | ✅ |
| `option.WithProject` | `ClientConfig::with_project()` | — | — | ✅ | ✅ |
| `option.WithWorkloadIdentity` | `ClientConfig::with_workload_identity()` | ✅ | ✅ | ✅ | ✅ |
| `auth.K8sServiceAccountTokenProvider` | `auth::K8sServiceAccountTokenProvider` | — | ✅ | ✅ | — |
| `auth.AzureManagedIdentityTokenProvider` | `auth::AzureManagedIdentityTokenProvider` | ✅ | ✅ | ✅ | — |
| `auth.GCPIDTokenProvider` | `auth::GcpIdTokenProvider` | ✅ | ✅ | ✅ | — |
| `shared.OAuthErrorCode` | `shared::OAuthErrorCode` | — | ✅ | ✅ | — |
| `DefaultClientOptions` env defaults | `RequestConfig::default()` | — | — | ✅ | — |
| `option.WithWebhookSecret` | `ClientConfig::with_webhook_secret()` | — | — | ✅ | ✅ |
| `Client.Execute` | `Client::execute()` / `execute_raw()` | ✅ | ✅ | — | ✅ |
| `Client.Get/Post/Put/Patch/Delete` | `Client::get/post/put/patch/delete()` | ✅ | ✅ | — | ✅ |
| `option.RequestOption` retry/header/query/timeout/body/middleware/capture | `RequestOptions` / `Client::with_options()` | ✅ | — | ✅ | ✅ |
| `RawJSON()` / `JSON.ExtraFields` / field presence | `RawJsonExt` / `JsonField` | — | ✅ | ✅ | ✅ |
| Generated union helpers (`AsAny`, `AsVariant`, `ParamOf...`) | `as_any()` / `as_*()` / `param_of_*()` | ✅ | ✅ | ✅ | — |
| Generated SDK headers/retry metadata | automatic request headers | — | — | — | ✅ |

## Pagination / Auto-Paging

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `CursorPage.GetNextPage` | `CursorPage::next_page()` | ✅ | ✅ | ✅ | ✅ |
| `CursorPageAutoPager` | `CursorPage::into_stream()` | ✅ | ✅ | ✅ | ✅ |
| `ConversationCursorPage.GetNextPage` | `ConversationCursorPage::next_page()` | ✅ | ✅ | ✅ | ✅ |
| `ConversationCursorPageAutoPager` | `ConversationCursorPage::into_stream()` | ✅ | ✅ | ✅ | ✅ |

## Chat Completions

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `ChatCompletions.New` | `chat().completions().create()` | ✅ | ✅ | ✅ | ✅ |
| `ChatCompletions.NewStreaming` | `chat().completions().create_stream()` | ✅ | ✅ | ✅ | ✅ |

## Completions

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Completions.New` | `completions().create()` | ✅ | ✅ | ✅ | ✅ |
| `Completions.NewStreaming` | `completions().create_stream()` | ✅ | ✅ | ✅ | ✅ |

## Embeddings

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Embeddings.New` | `embeddings().create()` | ✅ | ✅ | ✅ | ✅ |

## Models

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Models.List` | `models().list()` | — | ✅ | ✅ | ✅ |
| `Models.Get` | `models().get()` | — | ✅ | ✅ | ✅ |
| `Models.Delete` | `models().delete()` | — | ✅ | ✅ | ✅ |

## Moderations

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Moderations.New` | `moderations().create()` | ✅ | ✅ | ✅ | ✅ |

## Files

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Files.New` | `files().create()` | ✅ | ✅ | ✅ | ✅ |
| `Files.List` | `files().list()` | — | ✅ | ✅ | ✅ |
| `Files.Get` | `files().get()` | — | ✅ | ✅ | ✅ |
| `Files.Delete` | `files().delete()` | — | ✅ | ✅ | ✅ |
| `Files.Content` | `files().content()` | — | ✅ | ✅ | ✅ |

## Uploads

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Uploads.New` | `uploads().create()` | ✅ | ✅ | ✅ | ✅ |
| `Uploads.Parts.New` | `uploads().create_part()` | ✅ | ✅ | ✅ | ✅ |
| `Uploads.Complete` | `uploads().complete()` | ✅ | ✅ | ✅ | ✅ |
| `Uploads.Cancel` | `uploads().cancel()` | — | ✅ | ✅ | ✅ |
| `Uploads.Get` | `uploads().get()` | — | ✅ | ✅ | ✅ |

## Audio

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Audio.Transcriptions.New` | `audio().transcribe()` | ✅ | ✅ | ✅ | ✅ |
| `Audio.Transcriptions.NewStreaming` | `audio().transcribe_streaming()` | ✅ | ✅ | ✅ | ✅ |
| `Audio.Translations.New` | `audio().translate()` | ✅ | ✅ | ✅ | ✅ |
| `Audio.Speech.New` | `audio().speech()` | ✅ | ✅ | ✅ | ✅ |

## Images

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Images.Generations.New` | `images().generate()` | ✅ | ✅ | ✅ | ✅ |
| `Images.Generations.NewStreaming` | `images().generate_streaming()` | ✅ | ✅ | ✅ | ✅ |
| `Images.Edits.New` | `images().edit()` | ✅ | ✅ | ✅ | ✅ |
| `Images.Edits.NewStreaming` | `images().edit_streaming()` | ✅ | ✅ | ✅ | ✅ |
| `Images.Variations.New` | `images().create_variation()` | ✅ | ✅ | ✅ | ✅ |

## Fine-Tuning

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `FineTuning.Jobs.New` | `fine_tuning().jobs().create()` | ✅ | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.Get` | `fine_tuning().jobs().get()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.List` | `fine_tuning().jobs().list()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.Cancel` | `fine_tuning().jobs().cancel()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.ListEvents` | `fine_tuning().jobs().list_events()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.Pause` | `fine_tuning().jobs().pause()` | — | ✅ | ✅ | ✅ |
| `FineTuning.Jobs.Resume` | `fine_tuning().jobs().resume()` | — | ✅ | ✅ | ✅ |

## Batches

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Batches.New` | `batches().create()` | ✅ | ✅ | ✅ | ✅ |
| `Batches.Get` | `batches().get()` | — | ✅ | ✅ | ✅ |
| `Batches.List` | `batches().list()` | — | ✅ | ✅ | ✅ |
| `Batches.Cancel` | `batches().cancel()` | — | ✅ | ✅ | ✅ |

## Vector Stores

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `VectorStores.New` | `vector_stores().create()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Get` | `vector_stores().get()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Update` | `vector_stores().update()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.List` | `vector_stores().list()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Delete` | `vector_stores().delete()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Search` | `vector_stores().search()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.FileBatches.New` | `vector_stores().file_batches().create()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.FileBatches.Get` | `vector_stores().file_batches().get()` | — | ✅ | ✅ | ✅ |
| `VectorStores.FileBatches.Cancel` | `vector_stores().file_batches().cancel()` | — | ✅ | ✅ | ✅ |
| `VectorStores.FileBatches.ListFiles` | `vector_stores().file_batches().list_files()` | — | ✅ | ✅ | ✅ |
| `VectorStores.FileBatches.PollStatus` | `vector_stores().file_batches().poll_status()` | — | ✅ | ✅ | ✅ |
| `VectorStores.FileBatches.NewAndPoll` | `vector_stores().file_batches().create_and_poll()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.FileBatches.UploadAndPoll` | `vector_stores().file_batches().upload_and_poll()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Files.New` | `vector_stores().files().create()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Files.NewAndPoll` | `vector_stores().files().create_and_poll()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Files.Upload` | `vector_stores().files().upload()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Files.UploadAndPoll` | `vector_stores().files().upload_and_poll()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Files.Get` | `vector_stores().files().get()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Files.Update` | `vector_stores().files().update()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Files.List` | `vector_stores().files().list()` | ✅ | ✅ | ✅ | ✅ |
| `VectorStores.Files.Delete` | `vector_stores().files().delete()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Files.Content` | `vector_stores().files().content()` | — | ✅ | ✅ | ✅ |
| `VectorStores.Files.PollStatus` | `vector_stores().files().poll_status()` | — | ✅ | ✅ | ✅ |

## Graders

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Graders.Namespace` | `graders().grader_models()` | ✅ | ✅ | ✅ | ✅ |

## Beta / Assistants

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
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

## Beta / ChatKit

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Beta.ChatKit.Sessions.Create` | `beta().chat_kit().sessions().create()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.ChatKit.Sessions.Cancel` | `beta().chat_kit().sessions().cancel()` | — | ✅ | ✅ | ✅ |
| `Beta.ChatKit.Threads.Get` | `beta().chat_kit().threads().get()` | — | ✅ | ✅ | ✅ |
| `Beta.ChatKit.Threads.List` | `beta().chat_kit().threads().list()` | ✅ | ✅ | ✅ | ✅ |
| `Beta.ChatKit.Threads.Delete` | `beta().chat_kit().threads().delete()` | — | ✅ | ✅ | ✅ |
| `Beta.ChatKit.Threads.ListItems` | `beta().chat_kit().threads().list_items()` | ✅ | ✅ | ✅ | ✅ |

## Responses API

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Responses.New` | `responses().create()` | ✅ | ✅ | ✅ | ✅ |
| `Responses.NewStreaming` | `responses().create_stream()` | ✅ | ✅ | ✅ | ✅ |
| `Responses.Get` | `responses().get()` | ✅ | ✅ | ✅ | ✅ |
| `Responses.GetStreaming` | `responses().get_stream()` | ✅ | ✅ | ✅ | ✅ |
| `Responses.Delete` | `responses().delete()` | — | ✅ | ✅ | ✅ |
| `Responses.Cancel` | `responses().cancel()` | — | ✅ | ✅ | ✅ |
| `Responses.Compact` | `responses().compact()` | ✅ | ✅ | ✅ | ✅ |
| `Responses.InputItems.List` | `responses().list_input_items()` | ✅ | ✅ | ✅ | ✅ |
| `Responses.InputTokens.Count` | `responses().input_tokens().count()` | ✅ | ✅ | ✅ | ✅ |

## Realtime

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Realtime.Sessions.New` | `realtime().client_secrets().create()` | ✅ | ✅ | ✅ | ✅ |
| `Realtime.Calls.Accept` | `realtime().calls().accept()` | ✅ | ✅ | ✅ | ✅ |
| `Realtime.Calls.Hangup` | `realtime().calls().hangup()` | ✅ | ✅ | ✅ | ✅ |
| `Realtime.Calls.Refer` | `realtime().calls().refer()` | ✅ | ✅ | ✅ | ✅ |
| `Realtime.Calls.Reject` | `realtime().calls().reject()` | ✅ | ✅ | ✅ | ✅ |

## Webhooks

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Webhooks.VerifySignature` | `webhooks().verify_signature()` | — | — | ✅ | ✅ |
| `Webhooks.Unwrap` | `webhooks().unwrap_event()` | — | ✅ | ✅ | ✅ |

## Conversations

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Conversations.New` | `conversations().create()` | ✅ | ✅ | ✅ | ✅ |
| `Conversations.Get` | `conversations().get()` | — | ✅ | ✅ | ✅ |
| `Conversations.Update` | `conversations().update()` | ✅ | ✅ | ✅ | ✅ |
| `Conversations.Delete` | `conversations().delete()` | — | ✅ | ✅ | ✅ |
| `Conversations.Items.New` | `conversations().items().create()` | ✅ | ✅ | ✅ | ✅ |
| `Conversations.Items.Get` | `conversations().items().get()` | ✅ | ✅ | ✅ | ✅ |
| `Conversations.Items.List` | `conversations().items().list()` | ✅ | ✅ | ✅ | ✅ |
| `Conversations.Items.Delete` | `conversations().items().delete()` | — | ✅ | ✅ | ✅ |

## Containers

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Containers.New` | `containers().create()` | ✅ | ✅ | ✅ | ✅ |
| `Containers.Get` | `containers().get()` | — | ✅ | ✅ | ✅ |
| `Containers.List` | `containers().list()` | ✅ | ✅ | ✅ | ✅ |
| `Containers.Delete` | `containers().delete()` | — | ✅ | ✅ | ✅ |
| `ContainerFiles.New` | `containers().files().create()` | ✅ | ✅ | ✅ | ✅ |
| `ContainerFiles.Get` | `containers().files().get()` | — | ✅ | ✅ | ✅ |
| `ContainerFiles.List` | `containers().files().list()` | ✅ | ✅ | ✅ | ✅ |
| `ContainerFiles.Delete` | `containers().files().delete()` | — | ✅ | ✅ | ✅ |
| `ContainerFileContent.Get` | `containers().files().content().get()` | — | ✅ | ✅ | ✅ |

## Skills

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Skills.New` | `skills().create()` | ✅ | ✅ | ✅ | ✅ |
| `Skills.Get` | `skills().get()` | — | ✅ | ✅ | ✅ |
| `Skills.Update` | `skills().update()` | ✅ | ✅ | ✅ | ✅ |
| `Skills.List` | `skills().list()` | ✅ | ✅ | ✅ | ✅ |
| `Skills.Delete` | `skills().delete()` | — | ✅ | ✅ | ✅ |
| `Skills.Versions.New` | `skills().versions().create()` | ✅ | ✅ | ✅ | ✅ |
| `Skills.Versions.Get` | `skills().versions().get()` | — | ✅ | ✅ | ✅ |
| `Skills.Versions.List` | `skills().versions().list()` | ✅ | ✅ | ✅ | ✅ |
| `Skills.Versions.Delete` | `skills().versions().delete()` | — | ✅ | ✅ | ✅ |

## Videos

| Go Method | Rust Method | Request Struct | Response Struct | Unit Test | Integration Test |
|-----------|-------------|----------------|-----------------|-----------|------------------|
| `Videos.New` | `videos().create()` | ✅ | ✅ | ✅ | ✅ |
| `Videos.Get` | `videos().get()` | — | ✅ | ✅ | ✅ |
| `Videos.List` | `videos().list()` | ✅ | ✅ | ✅ | ✅ |
| `Videos.Delete` | `videos().delete()` | — | ✅ | ✅ | ✅ |
| `Videos.Characters.New` | `videos().create_character()` | ✅ | ✅ | ✅ | ✅ |
| `Videos.Characters.Get` | `videos().get_character()` | — | ✅ | ✅ | ✅ |
| `Videos.DownloadContent` | `videos().download_content()` | ✅ | ✅ | ✅ | ✅ |
| `Videos.Edit` | `videos().edit()` | ✅ | ✅ | ✅ | ✅ |
| `Videos.Extend` | `videos().extend()` | ✅ | ✅ | ✅ | ✅ |
| `Videos.Remix` | `videos().remix()` | ✅ | ✅ | ✅ | ✅ |
| `Videos.PollStatus` | `videos().poll_status()` | — | ✅ | ✅ | ✅ |
| `Videos.NewAndPoll` | `videos().create_and_poll()` | ✅ | ✅ | ✅ | ✅ |

## Examples

| Example | Status |
|---------|--------|
| `chat_completion` | ✅ |
| `chat_completion_streaming` | ✅ |
| `chat_completion_tool_calling` | ✅ |
| `audio_transcription` | ✅ |
| `audio_text_to_speech` | ✅ |
| `embeddings` | ✅ |
| `fine_tuning` | ✅ |
| `image_generation` | ✅ |
| `structured_outputs` | ✅ |
| `responses` | ✅ |
| `responses_streaming` | ✅ |
| `video_generation` | ✅ |
| `files_upload` | ✅ |
| `batches` | ✅ |
| `models_list` | ✅ |
| `moderations` | ✅ |
| `vector_stores` | ✅ |

## Remaining Rewrite TODOs

- [x] Preserve raw JSON metadata and unknown fields for decoded responses via `RawJsonExt`, `JsonMetadata`, and `JsonField` field-presence metadata.
- [x] Add request-option parity primitives: custom HTTP client, async middleware, header/query set/add/delete, JSON mutation, request body override, response capture, and scoped `Client::with_options()`.
- [x] Expose Rust-named generated union helpers on SDK union enums: `as_any()`, variant accessors/predicates, and `param_of_*()` constructors.
