<template>
  <q-page class="q-pa-md">
    <div
      class="settings-container"
      style="max-width: 900px; margin: 0 auto;"
    >
      <div class="row items-center q-mb-md">
        <q-btn
          flat
          dense
          icon="arrow_back"
          label="Back"
          class="q-mr-md"
          @click="goBack"
        />
        <h1 class="text-h4 q-my-none">
          Settings
        </h1>
      </div>

      <!-- Loading state -->
      <q-linear-progress
        v-if="settingsStore.loading"
        indeterminate
        color="primary"
        class="q-mb-md"
      />

      <!-- Error notification -->
      <q-banner
        v-if="settingsStore.hasError"
        class="bg-negative text-white q-mb-md"
        dense
      >
        <template #avatar>
          <q-icon
            name="error"
            color="white"
          />
        </template>
        {{ settingsStore.error }}
        <template #action>
          <q-btn
            flat
            label="Dismiss"
            @click="settingsStore.clearError()"
          />
        </template>
      </q-banner>

      <!-- General Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="settings"
              class="q-mr-sm"
            />
            General
          </div>

          <div class="q-gutter-md">
            <q-input
              v-model="localSettings.default_save_path"
              label="Sessions Root Folder"
              hint="Default folder to save QA sessions"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="folder" />
              </template>
              <template #append>
                <q-btn
                  round
                  dense
                  flat
                  icon="folder_open"
                  @click="selectSessionsRoot"
                >
                  <q-tooltip>Browse</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-toggle
              v-model="localSettings.launch_on_startup"
              label="Launch on Windows startup"
              color="primary"
            />

            <q-toggle
              v-model="localSettings.minimize_to_tray"
              label="Minimize to tray on close"
              color="primary"
            />

            <q-toggle
              v-model="localSettings.show_status_widget"
              label="Auto-open status window when session starts"
              color="primary"
            >
              <q-tooltip>
                Automatically opens the floating status bar when a session starts.
                You can also open it manually from the toolbar button.
              </q-tooltip>
            </q-toggle>
          </div>
        </q-card-section>
      </q-card>

      <!-- Hotkeys Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="keyboard"
              class="q-mr-sm"
            />
            Hotkeys
          </div>

          <div class="q-gutter-md">
            <q-input
              v-model="localSettings.hotkey_toggle_session"
              label="Start/End Session"
              hint="Default: Ctrl+Alt+S"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="power_settings_new" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_toggle_session')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_new_bug"
              label="New Bug Capture"
              hint="Default: Ctrl+Alt+B"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="camera" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_new_bug')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_end_bug"
              label="End Bug Capture"
              hint="Default: Ctrl+Alt+E"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="stop" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_end_bug')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_quick_notepad"
              label="Quick Notepad"
              hint="Default: Ctrl+Alt+N"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="note_add" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_quick_notepad')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-input
              v-model="localSettings.hotkey_session_notepad"
              label="Session Notepad"
              hint="Default: Ctrl+Alt+P"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="notes" />
              </template>
              <template #append>
                <q-btn
                  flat
                  dense
                  label="Record"
                  color="primary"
                  @click="recordHotkey('hotkey_session_notepad')"
                >
                  <q-tooltip>Click to record a new hotkey</q-tooltip>
                </q-btn>
              </template>
            </q-input>

            <q-banner
              v-if="hotkeyConflict"
              class="bg-warning text-white"
              dense
            >
              <template #avatar>
                <q-icon
                  name="warning"
                  color="white"
                />
              </template>
              {{ hotkeyConflict }}
            </q-banner>
          </div>
        </q-card-section>
      </q-card>

      <!-- Annotation Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="draw"
              class="q-mr-sm"
            />
            Annotation
          </div>

          <div class="q-gutter-md">
            <q-toggle
              v-model="localSettings.annotation_auto_open"
              label="Auto-open annotation on screenshot capture"
              color="primary"
            />

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Save Mode
              </div>
              <q-option-group
                v-model="localSettings.annotation_save_mode"
                :options="annotationSaveModeOptions"
                color="primary"
              />
            </div>

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Default Color
              </div>
              <q-btn-group flat>
                <q-btn
                  v-for="color in annotationColors"
                  :key="color.value"
                  :style="{backgroundColor: color.value, color: 'white'}"
                  :outline="localSettings.annotation_default_color !== color.value"
                  :unelevated="localSettings.annotation_default_color === color.value"
                  @click="localSettings.annotation_default_color = color.value"
                >
                  {{ color.label }}
                </q-btn>
              </q-btn-group>
            </div>

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Default Stroke Width
              </div>
              <q-option-group
                v-model="localSettings.annotation_stroke_width"
                :options="strokeWidthOptions"
                color="primary"
              />
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- QA Profiles Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="manage_accounts"
              class="q-mr-sm"
            />
            QA Profiles
          </div>

          <!-- Active Profile Selector -->
          <div class="q-mb-md">
            <q-select
              :model-value="profileStore.activeProfileId"
              :options="profileSelectOptions"
              label="Active Profile"
              hint="Select the active QA profile for this session"
              outlined
              emit-value
              map-options
              clearable
              @update:model-value="onActiveProfileChange"
            >
              <template #prepend>
                <q-icon name="person" />
              </template>
            </q-select>
          </div>

          <!-- Profile List -->
          <div
            v-if="profileStore.profiles.length > 0"
            class="q-mb-md"
          >
            <div class="text-subtitle2 q-mb-sm">
              Profiles
            </div>
            <q-list
              bordered
              separator
            >
              <q-item
                v-for="profile in profileStore.profiles"
                :key="profile.id"
                class="q-pa-sm"
              >
                <q-item-section>
                  <q-item-label>{{ profile.name }}</q-item-label>
                  <q-item-label caption>
                    {{ profile.custom_fields.length }} custom field{{ profile.custom_fields.length !== 1 ? 's' : '' }}
                    <span v-if="profile.linear_config"> · Linear configured</span>
                    <span v-else> · No Linear config</span>
                    <span v-if="profile.id === profileStore.activeProfileId"> · Active</span>
                  </q-item-label>
                </q-item-section>
                <q-item-section side>
                  <div class="row q-gutter-xs">
                    <q-btn
                      flat
                      dense
                      icon="edit"
                      color="primary"
                      @click="openEditProfile(profile)"
                    >
                      <q-tooltip>Edit profile</q-tooltip>
                    </q-btn>
                    <q-btn
                      flat
                      dense
                      icon="delete"
                      color="negative"
                      @click="confirmDeleteProfile(profile.id, profile.name)"
                    >
                      <q-tooltip>Delete profile</q-tooltip>
                    </q-btn>
                  </div>
                </q-item-section>
              </q-item>
            </q-list>
          </div>

          <div
            v-else
            class="text-grey-6 q-mb-md"
          >
            No profiles yet. Create one to define custom fields, Linear config, and title conventions.
          </div>

          <!-- Create Profile Button -->
          <q-btn
            outline
            color="primary"
            icon="add"
            label="New Profile"
            @click="openCreateProfile"
          />
        </q-card-section>
      </q-card>

      <!-- Profile Create/Edit Dialog -->
      <q-dialog
        v-model="profileDialogOpen"
        persistent
        maximized
      >
        <q-card style="max-width: 700px; width: 100%; margin: 1rem auto; max-height: calc(100vh - 2rem); overflow-y: auto;">
          <q-card-section class="row items-center q-pb-none">
            <div class="text-h6">
              {{ editingProfile ? 'Edit Profile' : 'New Profile' }}
            </div>
            <q-space />
            <q-btn
              v-close-popup
              icon="close"
              flat
              round
              dense
            />
          </q-card-section>

          <q-card-section>
            <!-- Profile Name -->
            <q-input
              v-model="profileForm.name"
              label="Profile Name"
              outlined
              dense
              class="q-mb-md"
              :rules="[val => !!val || 'Profile name is required']"
            >
              <template #prepend>
                <q-icon name="label" />
              </template>
            </q-input>

            <!-- Linear Config Section -->
            <q-expansion-item
              label="Linear Configuration"
              icon="integration_instructions"
              class="q-mb-md"
              header-class="text-subtitle2"
            >
              <q-card flat>
                <q-card-section class="q-gutter-sm">
                  <q-select
                    v-if="linearTeamOptions.length > 0"
                    v-model="profileForm.linearTeamId"
                    :options="linearTeamOptions"
                    label="Team"
                    hint="Select Linear team for this profile"
                    outlined
                    dense
                    emit-value
                    map-options
                  >
                    <template #prepend>
                      <q-icon name="group" />
                    </template>
                  </q-select>
                  <q-input
                    v-else
                    v-model="profileForm.linearTeamId"
                    label="Team ID"
                    hint="Test connection in Linear settings to load teams, or enter team ID manually"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="group" />
                    </template>
                  </q-input>

                  <q-input
                    v-model="profileForm.linearApiKey"
                    label="API Key"
                    type="password"
                    hint="Linear API key for this profile"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="vpn_key" />
                    </template>
                  </q-input>

                  <q-input
                    v-model="profileForm.linearDefaultAssigneeId"
                    label="Default Assignee ID"
                    hint="Linear user ID to assign issues by default"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="person_outline" />
                    </template>
                  </q-input>

                  <q-input
                    v-model="profileForm.linearDefaultBugLabelIds"
                    label="Default Bug Label IDs"
                    hint="Comma-separated label IDs for bugs"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="bug_report" />
                    </template>
                  </q-input>

                  <q-input
                    v-model="profileForm.linearDefaultFeatureLabelIds"
                    label="Default Feature Label IDs"
                    hint="Comma-separated label IDs for features"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="star_outline" />
                    </template>
                  </q-input>

                  <q-input
                    v-model="profileForm.linearDefaultStateId"
                    label="Default State ID"
                    hint="Linear workflow state ID for new issues"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="radio_button_unchecked" />
                    </template>
                  </q-input>
                </q-card-section>
              </q-card>
            </q-expansion-item>

            <!-- Title Conventions Section -->
            <q-expansion-item
              label="Title Conventions"
              icon="title"
              class="q-mb-md"
              header-class="text-subtitle2"
            >
              <q-card flat>
                <q-card-section class="q-gutter-sm">
                  <q-input
                    v-model="profileForm.titleBugPrefix"
                    label="Bug Prefix"
                    hint="Prefix for bug ticket titles (e.g. [BUG])"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="bug_report" />
                    </template>
                  </q-input>

                  <q-input
                    v-model="profileForm.titleFeaturePrefix"
                    label="Feature Prefix"
                    hint="Prefix for feature ticket titles (e.g. [FEAT])"
                    outlined
                    dense
                  >
                    <template #prepend>
                      <q-icon name="star_outline" />
                    </template>
                  </q-input>
                </q-card-section>
              </q-card>
            </q-expansion-item>

            <!-- Area Categories Section -->
            <q-expansion-item
              label="Area Categories"
              icon="category"
              class="q-mb-md"
              header-class="text-subtitle2"
            >
              <q-card flat>
                <q-card-section>
                  <div
                    v-for="(cat, index) in profileForm.areaCategories"
                    :key="index"
                    class="row q-gutter-sm q-mb-sm items-center"
                  >
                    <q-input
                      v-model="cat.code"
                      label="Code"
                      outlined
                      dense
                      class="col-2"
                    />
                    <q-input
                      v-model="cat.name"
                      label="Name"
                      outlined
                      dense
                      class="col"
                    />
                    <q-input
                      v-model="cat.description"
                      label="Description"
                      outlined
                      dense
                      class="col"
                    />
                    <q-btn
                      flat
                      dense
                      icon="remove_circle_outline"
                      color="negative"
                      @click="removeAreaCategory(index)"
                    >
                      <q-tooltip>Remove category</q-tooltip>
                    </q-btn>
                  </div>
                  <q-btn
                    outline
                    dense
                    icon="add"
                    label="Add Category"
                    color="primary"
                    class="q-mt-sm"
                    @click="addAreaCategory"
                  />
                </q-card-section>
              </q-card>
            </q-expansion-item>

            <!-- Custom Fields Section -->
            <q-expansion-item
              label="Custom Metadata Fields"
              icon="tune"
              class="q-mb-md"
              header-class="text-subtitle2"
            >
              <q-card flat>
                <q-card-section>
                  <div
                    v-for="(field, index) in profileForm.customFields"
                    :key="index"
                    class="q-mb-md"
                    style="border: 1px solid rgba(0,0,0,0.12); border-radius: 4px; padding: 8px;"
                  >
                    <div class="row q-gutter-sm q-mb-sm">
                      <q-input
                        v-model="field.key"
                        label="Key"
                        outlined
                        dense
                        class="col"
                      />
                      <q-input
                        v-model="field.label"
                        label="Label"
                        outlined
                        dense
                        class="col"
                      />
                    </div>
                    <div class="row q-gutter-sm items-center">
                      <q-select
                        v-model="field.field_type"
                        :options="customFieldTypeOptions"
                        label="Type"
                        outlined
                        dense
                        emit-value
                        map-options
                        class="col-3"
                      />
                      <q-input
                        v-model="field.default_value"
                        label="Default Value"
                        outlined
                        dense
                        class="col"
                      />
                      <q-toggle
                        v-model="field.required"
                        label="Required"
                        color="primary"
                        class="col-auto"
                      />
                      <q-btn
                        flat
                        dense
                        icon="remove_circle_outline"
                        color="negative"
                        class="col-auto"
                        @click="removeCustomField(index)"
                      >
                        <q-tooltip>Remove field</q-tooltip>
                      </q-btn>
                    </div>
                  </div>
                  <q-btn
                    outline
                    dense
                    icon="add"
                    label="Add Field"
                    color="primary"
                    class="q-mt-sm"
                    @click="addCustomField"
                  />
                </q-card-section>
              </q-card>
            </q-expansion-item>
          </q-card-section>

          <q-card-actions align="right">
            <q-btn
              flat
              label="Cancel"
              v-close-popup
            />
            <q-btn
              color="primary"
              label="Save Profile"
              :loading="profileStore.loading"
              :disable="!profileForm.name.trim()"
              @click="saveProfile"
            />
          </q-card-actions>
        </q-card>
      </q-dialog>

      <!-- AI (Claude) Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="smart_toy"
              class="q-mr-sm"
            />
            AI (Claude)
          </div>

          <div class="q-gutter-md">
            <q-banner
              :class="claudeStatus === 'available' ? 'bg-positive' : 'bg-warning'"
              class="text-white"
              dense
            >
              <template #avatar>
                <q-icon
                  :name="claudeStatus === 'available' ? 'check_circle' : 'info'"
                  color="white"
                />
              </template>
              <div v-if="claudeStatus === 'available'">
                Connected via Claude Code (uses your Claude subscription)
              </div>
              <div v-else-if="claudeStatus === 'not_found'">
                Claude Code not found. Install Claude Code and sign in to enable AI features with your Claude subscription.
              </div>
              <div v-else>
                Checking Claude Code status...
              </div>
            </q-banner>

            <q-btn
              outline
              color="primary"
              label="Refresh Status"
              icon="refresh"
              :loading="testingClaude"
              @click="testClaudeConnection"
            />

            <q-toggle
              v-model="localSettings.ai_auto_generate"
              label="Auto-generate descriptions on review"
              color="primary"
              :disable="claudeStatus !== 'available'"
            />
          </div>
        </q-card-section>
      </q-card>

      <!-- Ticketing Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="integration_instructions"
              class="q-mr-sm"
            />
            Ticketing
          </div>

          <div class="q-gutter-md">
            <q-select
              v-model="localSettings.ticketing_provider"
              :options="ticketingProviderOptions"
              label="Integration Type"
              outlined
              emit-value
              map-options
            >
              <template #prepend>
                <q-icon name="cloud" />
              </template>
            </q-select>

            <q-select
              v-model="localSettings.default_bug_type"
              :options="bugTypeOptions"
              label="Default Bug Type"
              outlined
              emit-value
              map-options
            >
              <template #prepend>
                <q-icon name="bug_report" />
              </template>
            </q-select>

            <!-- Linear API Configuration -->
            <div v-if="localSettings.ticketing_provider === 'linear'">
              <q-separator class="q-my-md" />
              <div class="text-subtitle2 q-mb-sm">
                Linear Configuration
              </div>

              <q-input
                v-model="localSettings.linear_api_key"
                label="Linear API Key"
                hint="Get your API key from https://linear.app/settings/api"
                type="password"
                outlined
                dense
                class="q-mb-md"
              >
                <template #prepend>
                  <q-icon name="vpn_key" />
                </template>
              </q-input>

              <q-select
                v-if="linearTeamOptions.length > 0"
                v-model="localSettings.linear_team_id"
                :options="linearTeamOptions"
                label="Team"
                hint="Select your Linear team"
                outlined
                dense
                emit-value
                map-options
                :loading="fetchingLinearTeams"
                :readonly="linearTeamOptions.length === 1"
                class="q-mb-md"
              >
                <template #prepend>
                  <q-icon name="group" />
                </template>
              </q-select>
              <q-input
                v-else
                v-model="localSettings.linear_team_id"
                label="Team ID"
                :hint="fetchingLinearTeams ? 'Fetching teams...' : 'Test connection to load teams, or enter team ID manually'"
                outlined
                dense
                :loading="fetchingLinearTeams"
                class="q-mb-md"
              >
                <template #prepend>
                  <q-icon name="group" />
                </template>
              </q-input>

              <q-btn
                outline
                color="primary"
                label="Test Connection"
                icon="check_circle"
                :loading="testingLinearConnection"
                :disable="!localSettings.linear_api_key"
                @click="testLinearConnection"
              />
            </div>

            <q-separator class="q-my-md" />

            <q-input
              v-model="localSettings.linear_config_path"
              label="Linear Project Configuration (Optional)"
              hint="Path to Linear configuration file (advanced users only)"
              outlined
              readonly
            >
              <template #prepend>
                <q-icon name="settings_applications" />
              </template>
              <template #append>
                <q-btn
                  round
                  dense
                  flat
                  icon="folder_open"
                  @click="selectLinearConfigPath"
                >
                  <q-tooltip>Browse</q-tooltip>
                </q-btn>
                <q-btn
                  v-if="localSettings.linear_config_path"
                  round
                  dense
                  flat
                  icon="clear"
                  @click="localSettings.linear_config_path = ''"
                >
                  <q-tooltip>Clear</q-tooltip>
                </q-btn>
              </template>
            </q-input>
          </div>
        </q-card-section>
      </q-card>

      <!-- Ticket Template Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="description"
              class="q-mr-sm"
            />
            Ticket Template
          </div>

          <div class="q-gutter-md">
            <div class="row items-center">
              <div class="col-4 text-grey-7">
                Template Source
              </div>
              <div class="col">
                {{ templateSource }}
              </div>
            </div>

            <div class="row q-gutter-sm">
              <q-btn
                outline
                color="primary"
                label="Edit Template"
                icon="edit"
                @click="openTemplateEditor"
              />
              <q-btn
                outline
                color="warning"
                label="Reset to Default"
                icon="restore"
                @click="confirmResetTemplate"
              />
            </div>

            <q-separator />

            <div>
              <div class="text-subtitle2 q-mb-sm">
                Live Preview
              </div>
              <q-card
                flat
                bordered
                class="bg-grey-1"
              >
                <q-card-section>
                  <div
                    v-if="templatePreview"
                    class="template-preview"
                    style="white-space: pre-wrap; font-family: monospace; font-size: 12px;"
                  >
                    {{ templatePreview }}
                  </div>
                  <div
                    v-else
                    class="text-grey-6"
                  >
                    Loading preview...
                  </div>
                </q-card-section>
              </q-card>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- Swarm Integration Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="bug_report"
              class="q-mr-sm"
            />
            Swarm Integration
          </div>

          <div class="q-gutter-md">
            <q-input
              v-model="localSettings.swarm_ticket_db_path"
              label="Swarm Ticket DB Path"
              hint="Path to the swarm ticket database used for dogfooding exports (default: .swarm/tickets/tickets.db)"
              outlined
              dense
            >
              <template #prepend>
                <q-icon name="storage" />
              </template>
            </q-input>
          </div>
        </q-card-section>
      </q-card>

      <!-- About Section -->
      <q-card class="q-mb-md">
        <q-card-section>
          <div class="text-h6 q-mb-md">
            <q-icon
              name="info"
              class="q-mr-sm"
            />
            About
          </div>

          <div class="q-gutter-md">
            <div class="row items-center">
              <div class="col-4 text-grey-7">
                App Version
              </div>
              <div class="col">
                {{ appVersion }}
              </div>
            </div>

            <div class="row items-center">
              <div class="col-4 text-grey-7">
                Developer
              </div>
              <div class="col">
                Unbroken Technology
              </div>
            </div>

            <q-separator />

            <div class="row q-gutter-sm">
              <q-btn
                flat
                color="primary"
                label="Website"
                icon="public"
                @click="openLink('https://unbroken.tech')"
              />
              <q-btn
                flat
                color="primary"
                label="Support"
                icon="help"
                @click="openLink('mailto:support@unbroken.tech')"
              />
              <q-btn
                flat
                color="primary"
                label="Changelog"
                icon="history"
                @click="openLink('https://github.com/UnbrokenTechnology/unbroken_qa_capture/releases')"
              />
            </div>

            <div class="text-caption text-grey-6">
              Licensed under MIT License
            </div>
          </div>
        </q-card-section>
      </q-card>

      <!-- Action Buttons -->
      <div class="row q-gutter-md justify-end">
        <q-btn
          outline
          color="grey-7"
          label="Reset to Defaults"
          @click="confirmReset"
        />
        <q-btn
          outline
          color="grey-7"
          label="Cancel"
          @click="cancelChanges"
        />
        <q-btn
          color="primary"
          label="Save Settings"
          :disable="!hasValidSettings || settingsStore.loading"
          @click="saveSettings"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useProfileStore } from '@/stores/profile'
import { useQuasar } from 'quasar'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import { useRouter } from 'vue-router'
import { getClaudeStatus, refreshClaudeStatus, ticketingFetchTeams } from '@/api/tauri'
import type { QaProfile, AreaCategory, CustomMetadataField, CustomFieldType, LinearTeam } from '@/types/backend'

const settingsStore = useSettingsStore()
const profileStore = useProfileStore()
const $q = useQuasar()
const router = useRouter()

// Local settings state (for editing before save)
const localSettings = ref({
  // General
  default_save_path: '',
  launch_on_startup: false,
  minimize_to_tray: true,
  show_status_widget: false,

  // Hotkeys
  hotkey_toggle_session: 'Ctrl+Alt+S',
  hotkey_new_bug: 'Ctrl+Alt+B',
  hotkey_end_bug: 'Ctrl+Alt+E',
  hotkey_quick_notepad: 'Ctrl+Alt+N',
  hotkey_session_notepad: 'Ctrl+Alt+P',

  // Annotation
  annotation_auto_open: true,
  annotation_save_mode: 'alongside',
  annotation_default_color: '#FF0000',
  annotation_stroke_width: 'medium',

  // AI
  ai_auto_generate: false,

  // Ticketing
  ticketing_provider: 'linear',
  default_bug_type: 'bug',
  linear_api_key: '',
  linear_team_id: '',
  linear_config_path: '',

  // Swarm Integration
  swarm_ticket_db_path: '',
})

// UI state
const hotkeyConflict = ref<string | null>(null)
const claudeStatus = ref<'available' | 'not_found' | 'checking'>('checking')
const testingClaude = ref(false)
const testingLinearConnection = ref(false)
const linearTeams = ref<LinearTeam[]>([])
const fetchingLinearTeams = ref(false)
const appVersion = ref('1.0.0')
const templateSource = ref<string>('Default')
const templatePreview = ref<string>('')

// Options
const annotationSaveModeOptions = [
  { label: 'Save alongside original', value: 'alongside' },
  { label: 'Overwrite original', value: 'overwrite' },
]

const annotationColors = [
  { label: 'Red', value: '#FF0000' },
  { label: 'Blue', value: '#0000FF' },
  { label: 'Green', value: '#00FF00' },
  { label: 'Yellow', value: '#FFFF00' },
  { label: 'Orange', value: '#FFA500' },
  { label: 'Purple', value: '#800080' },
  { label: 'Black', value: '#000000' },
]

const strokeWidthOptions = [
  { label: 'Thin (2px)', value: 'thin' },
  { label: 'Medium (4px)', value: 'medium' },
  { label: 'Thick (8px)', value: 'thick' },
]

const ticketingProviderOptions = [
  { label: 'Linear', value: 'linear' },
  { label: 'File-based (Markdown)', value: 'file' },
]

const bugTypeOptions = [
  { label: 'Bug', value: 'bug' },
  { label: 'Feature', value: 'feature' },
  { label: 'Feedback', value: 'feedback' },
]

// Linear team dropdown options
const linearTeamOptions = computed(() =>
  linearTeams.value.map(t => ({ label: `${t.name} (${t.key})`, value: t.id }))
)

// Hotkey functions
function recordHotkey(key: string): void {
  $q.dialog({
    title: 'Record Hotkey',
    message: 'Press the key combination you want to use. (Note: This is a placeholder - actual hotkey recording will be implemented in the backend)',
    prompt: {
      model: '',
      type: 'text',
    },
    cancel: true,
  }).onOk((value: string) => {
    if (value) {
      // Check for conflicts
      const allHotkeys = [
        localSettings.value.hotkey_toggle_session,
        localSettings.value.hotkey_new_bug,
        localSettings.value.hotkey_end_bug,
        localSettings.value.hotkey_quick_notepad,
        localSettings.value.hotkey_session_notepad,
      ]

      if (allHotkeys.includes(value)) {
        hotkeyConflict.value = `Hotkey conflict: "${value}" is already assigned to another action`
        return
      }

      // Update the hotkey
      localSettings.value[key as keyof typeof localSettings.value] = value as never
      hotkeyConflict.value = null
    }
  })
}

// Check if all settings are valid
const hasValidSettings = computed(() => {
  return localSettings.value.default_save_path !== ''
})

// File path selection
async function selectSessionsRoot(): Promise<void> {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Sessions Root Folder',
    })
    if (selected) {
      localSettings.value.default_save_path = selected as string
    }
  } catch (err) {
    console.error('Failed to select sessions root:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to select folder',
    })
  }
}

async function selectLinearConfigPath(): Promise<void> {
  try {
    const selected = await open({
      multiple: false,
      title: 'Select Linear Configuration File',
      filters: [
        {
          name: 'JSON',
          extensions: ['json'],
        },
      ],
    })
    if (selected) {
      localSettings.value.linear_config_path = selected as string
    }
  } catch (err) {
    console.error('Failed to select Linear config:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to select file',
    })
  }
}

async function openTemplateEditor(): Promise<void> {
  try {
    await invoke('open_template_in_editor')
    $q.notify({
      type: 'positive',
      message: 'Template opened in system editor',
    })
  } catch (err) {
    console.error('Failed to open template editor:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to open template editor',
      caption: err instanceof Error ? err.message : String(err),
    })
  }
}

async function confirmResetTemplate(): Promise<void> {
  $q.dialog({
    title: 'Reset Template',
    message: 'Are you sure you want to reset the ticket template to the default? This will discard any customizations.',
    cancel: true,
    persistent: true,
  }).onOk(async () => {
    try {
      await invoke('reset_template_to_default')
      await loadTemplateInfo()
      $q.notify({
        type: 'positive',
        message: 'Template reset to default',
      })
    } catch (err) {
      console.error('Failed to reset template:', err)
      $q.notify({
        type: 'negative',
        message: 'Failed to reset template',
        caption: err instanceof Error ? err.message : String(err),
      })
    }
  })
}

async function loadTemplateInfo(): Promise<void> {
  try {
    // Get template path to determine if custom or default
    const templatePath = await invoke<string | null>('get_template_path')
    if (templatePath && templatePath.includes('custom_template.md')) {
      templateSource.value = 'Custom'
    } else {
      templateSource.value = 'Default'
    }

    // Load template preview
    const sampleBugData = {
      title: 'Sample Bug: Button Not Responding',
      bug_type: 'UI',
      description_steps: '1. Click the Submit button\\n2. Observe no response',
      description_expected: 'Button should trigger form submission',
      description_actual: 'Button does nothing when clicked',
      metadata: {
        meeting_id: 'MTG-2024-001',
        software_version: '2.5.0',
        environment: {
          os: 'Windows 11 Pro',
          display_resolution: '1920x1080',
          dpi_scaling: '150%',
          ram: '16GB',
          cpu: 'Intel Core i7-11800H',
          foreground_app: 'MyApp.exe',
        },
        console_captures: [],
        custom_fields: {},
      },
      folder_path: 'C:\\\\QA\\\\Sessions\\\\2024-02-16\\\\bug-001',
      captures: ['screenshot-01.png', 'screenshot-02.png'],
      console_output: 'Error: Form validation failed\\nUncaught TypeError: Cannot read property submit',
    }

    const preview = await invoke<string>('render_bug_template', { bugData: sampleBugData })
    templatePreview.value = preview
  } catch (err) {
    console.error('Failed to load template info:', err)
    templatePreview.value = 'Failed to load preview'
  }
}

// AI / Claude functions
async function checkClaudeStatus(): Promise<void> {
  try {
    claudeStatus.value = 'checking'
    const result = await getClaudeStatus()
    if (result.status === 'ready') {
      claudeStatus.value = 'available'
    } else {
      claudeStatus.value = 'not_found'
    }
  } catch (err) {
    console.error('Failed to check Claude status:', err)
    claudeStatus.value = 'not_found'
  }
}

async function testClaudeConnection(): Promise<void> {
  try {
    testingClaude.value = true
    const result = await refreshClaudeStatus()
    if (result.status === 'ready') {
      claudeStatus.value = 'available'
      $q.notify({
        type: 'positive',
        message: 'Claude Code connected! AI features use your Claude subscription.',
      })
    } else {
      claudeStatus.value = 'not_found'
      $q.notify({
        type: 'warning',
        message: 'Claude Code not found',
        caption: result.message ?? 'Install Claude Code and sign in to enable AI features.',
      })
    }
  } catch (err) {
    console.error('Failed to check Claude Code status:', err)
    claudeStatus.value = 'not_found'
    $q.notify({
      type: 'negative',
      message: 'Failed to check Claude Code status',
      caption: err instanceof Error ? err.message : String(err),
    })
  } finally {
    testingClaude.value = false
  }
}

// Linear connection test
async function testLinearConnection(): Promise<void> {
  if (!localSettings.value.linear_api_key) {
    $q.notify({
      type: 'warning',
      message: 'Please enter a Linear API key first',
    })
    return
  }

  testingLinearConnection.value = true
  try {
    // Test authentication with the provided credentials
    await invoke('ticketing_authenticate', {
      credentials: {
        api_key: localSettings.value.linear_api_key,
        team_id: localSettings.value.linear_team_id || null,
        workspace_id: null,
      },
    })

    // Fetch teams after successful authentication
    fetchingLinearTeams.value = true
    try {
      const teams = await ticketingFetchTeams()
      linearTeams.value = teams

      if (teams.length === 1) {
        // Auto-select the only team
        localSettings.value.linear_team_id = teams[0]?.id ?? ''
        $q.notify({
          type: 'positive',
          message: 'Linear connection successful!',
          caption: `Team "${teams[0]?.name}" auto-selected.`,
        })
      } else if (teams.length > 1) {
        $q.notify({
          type: 'positive',
          message: 'Linear connection successful!',
          caption: `${teams.length} teams found. Please select a team.`,
        })
      } else {
        $q.notify({
          type: 'positive',
          message: 'Linear connection successful!',
          caption: 'No teams found in your Linear workspace.',
        })
      }
    } catch (teamErr) {
      console.error('Failed to fetch Linear teams:', teamErr)
      $q.notify({
        type: 'positive',
        message: 'Linear connection successful!',
        caption: 'Could not fetch teams. You can enter the team ID manually.',
      })
    } finally {
      fetchingLinearTeams.value = false
    }
  } catch (err) {
    console.error('Linear connection test failed:', err)
    $q.notify({
      type: 'negative',
      message: 'Linear connection failed',
      caption: err instanceof Error ? err.message : String(err),
    })
  } finally {
    testingLinearConnection.value = false
  }
}

// About section functions
function openLink(url: string): void {
  openUrl(url).catch((err) => {
    console.error('Failed to open link:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to open link',
    })
  })
}

// Load settings from store
async function loadSettings(): Promise<void> {
  // Load hotkey config from backend
  let hotkeyConfig: any = null
  try {
    hotkeyConfig = await invoke('get_hotkey_config')
  } catch (err) {
    console.warn('Failed to load hotkey config from backend:', err)
  }

  localSettings.value = {
    // General
    default_save_path: settingsStore.getSetting('default_save_path', ''),
    launch_on_startup: settingsStore.getSetting('launch_on_startup', 'false') === 'true',
    minimize_to_tray: settingsStore.getSetting('minimize_to_tray', 'true') === 'true',
    show_status_widget: settingsStore.getSetting('show_status_widget', 'false') === 'true',

    // Hotkeys - load from backend HotkeyConfig if available
    hotkey_toggle_session: hotkeyConfig?.shortcuts?.toggle_session ?? 'Ctrl+Alt+S',
    hotkey_new_bug: hotkeyConfig?.shortcuts?.start_bug_capture ?? 'Ctrl+Alt+B',
    hotkey_end_bug: hotkeyConfig?.shortcuts?.end_bug_capture ?? 'Ctrl+Alt+E',
    hotkey_quick_notepad: hotkeyConfig?.shortcuts?.open_quick_notepad ?? 'Ctrl+Alt+N',
    hotkey_session_notepad: hotkeyConfig?.shortcuts?.open_session_notepad ?? 'Ctrl+Alt+P',

    // Annotation
    annotation_auto_open: settingsStore.getSetting('annotation_auto_open', 'true') === 'true',
    annotation_save_mode: settingsStore.getSetting('annotation_save_mode', 'alongside'),
    annotation_default_color: settingsStore.getSetting('annotation_default_color', '#FF0000'),
    annotation_stroke_width: settingsStore.getSetting('annotation_stroke_width', 'medium'),

    // AI
    ai_auto_generate: settingsStore.getSetting('ai_auto_generate', 'false') === 'true',

    // Ticketing
    ticketing_provider: settingsStore.getSetting('ticketing_provider', 'linear'),
    default_bug_type: settingsStore.getSetting('default_bug_type', 'bug'),
    linear_api_key: settingsStore.getSetting('linear_api_key', ''),
    linear_team_id: settingsStore.getSetting('linear_team_id', ''),
    linear_config_path: settingsStore.getSetting('linear_config_path', ''),

    // Swarm Integration
    swarm_ticket_db_path: settingsStore.getSetting('swarm_ticket_db_path', ''),
  }
}

// Save settings
async function saveSettings(): Promise<void> {
  try {
    // Save hotkey config to backend
    const hotkeyConfig = {
      shortcuts: {
        toggle_session: localSettings.value.hotkey_toggle_session,
        start_bug_capture: localSettings.value.hotkey_new_bug,
        end_bug_capture: localSettings.value.hotkey_end_bug,
        open_quick_notepad: localSettings.value.hotkey_quick_notepad,
        open_session_notepad: localSettings.value.hotkey_session_notepad,
      }
    }

    const hotkeyErrors = await invoke<string[]>('update_hotkey_config', { config: hotkeyConfig })
    if (hotkeyErrors && Array.isArray(hotkeyErrors) && hotkeyErrors.length > 0) {
      console.warn('Some hotkeys failed to register:', hotkeyErrors)
      $q.notify({
        type: 'warning',
        message: 'Some hotkeys could not be registered',
        caption: hotkeyErrors.join(', '),
      })
    }

    // Save all other settings to backend
    const settingsToSave: Record<string, string> = {
      // General
      default_save_path: localSettings.value.default_save_path,
      launch_on_startup: localSettings.value.launch_on_startup.toString(),
      minimize_to_tray: localSettings.value.minimize_to_tray.toString(),
      show_status_widget: localSettings.value.show_status_widget.toString(),

      // Annotation
      annotation_auto_open: localSettings.value.annotation_auto_open.toString(),
      annotation_save_mode: localSettings.value.annotation_save_mode,
      annotation_default_color: localSettings.value.annotation_default_color,
      annotation_stroke_width: localSettings.value.annotation_stroke_width,

      // AI (api key is saved via dedicated command below)
      ai_auto_generate: localSettings.value.ai_auto_generate.toString(),

      // Ticketing
      ticketing_provider: localSettings.value.ticketing_provider,
      default_bug_type: localSettings.value.default_bug_type,
      linear_api_key: localSettings.value.linear_api_key,
      linear_team_id: localSettings.value.linear_team_id,
      linear_config_path: localSettings.value.linear_config_path,

      // Swarm Integration
      swarm_ticket_db_path: localSettings.value.swarm_ticket_db_path,
    }

    // Save each setting
    for (const [key, value] of Object.entries(settingsToSave)) {
      await settingsStore.saveSetting(key, value)
    }

    // Save Linear credentials to ticketing table if API key is provided
    if (localSettings.value.linear_api_key && localSettings.value.ticketing_provider === 'linear') {
      try {
        await invoke('ticketing_save_credentials', {
          credentials: {
            api_key: localSettings.value.linear_api_key,
            team_id: localSettings.value.linear_team_id || null,
            workspace_id: null,
          },
        })
      } catch (err) {
        console.warn('Failed to save Linear credentials:', err)
      }
    }

    // If launch_on_startup changed, update Windows registry
    if (localSettings.value.launch_on_startup) {
      try {
        await invoke('enable_startup')
      } catch (err) {
        console.warn('Failed to enable startup:', err)
      }
    } else {
      try {
        await invoke('disable_startup')
      } catch (err) {
        console.warn('Failed to disable startup:', err)
      }
    }

    $q.notify({
      type: 'positive',
      message: 'Settings saved successfully',
      position: 'top',
    })
    goBack()
  } catch (err) {
    console.error('Failed to save settings:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to save settings',
      caption: err instanceof Error ? err.message : String(err),
    })
  }
}

// Cancel changes
async function cancelChanges(): Promise<void> {
  await loadSettings()
  $q.notify({
    type: 'info',
    message: 'Changes cancelled',
  })
}

// Navigate back
function goBack(): void {
  router.back()
}

// Escape key handler
function handleEscapeKey(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    goBack()
  }
}

// Reset to defaults
function confirmReset(): void {
  $q.dialog({
    title: 'Reset Settings',
    message: 'Are you sure you want to reset all settings to their default values?',
    cancel: true,
    persistent: true,
  }).onOk(async () => {
    settingsStore.resetToDefaults()
    await loadSettings()
    $q.notify({
      type: 'info',
      message: 'Settings reset to defaults',
    })
  })
}

// ============================================================================
// Profile Management
// ============================================================================

// Profile select options (for the active profile dropdown)
const profileSelectOptions = computed(() => {
  return profileStore.profiles.map(p => ({ label: p.name, value: p.id }))
})

// Custom field type options
const customFieldTypeOptions = [
  { label: 'Text', value: 'text' },
  { label: 'Number', value: 'number' },
  { label: 'Select', value: 'select' },
]

// Dialog state
const profileDialogOpen = ref(false)
const editingProfile = ref<QaProfile | null>(null)

// Form state for create/edit dialog
interface ProfileFormState {
  name: string
  linearTeamId: string
  linearApiKey: string
  linearDefaultAssigneeId: string
  linearDefaultBugLabelIds: string
  linearDefaultFeatureLabelIds: string
  linearDefaultStateId: string
  titleBugPrefix: string
  titleFeaturePrefix: string
  areaCategories: Array<{ code: string; name: string; description: string }>
  customFields: Array<{
    key: string
    label: string
    field_type: CustomFieldType
    default_value: string
    required: boolean
  }>
}

function makeEmptyProfileForm(): ProfileFormState {
  return {
    name: '',
    linearTeamId: '',
    linearApiKey: '',
    linearDefaultAssigneeId: '',
    linearDefaultBugLabelIds: '',
    linearDefaultFeatureLabelIds: '',
    linearDefaultStateId: '',
    titleBugPrefix: '',
    titleFeaturePrefix: '',
    areaCategories: [],
    customFields: [],
  }
}

const profileForm = ref<ProfileFormState>(makeEmptyProfileForm())

function openCreateProfile(): void {
  editingProfile.value = null
  profileForm.value = makeEmptyProfileForm()
  profileDialogOpen.value = true
}

function openEditProfile(profile: QaProfile): void {
  editingProfile.value = profile
  profileForm.value = {
    name: profile.name,
    linearTeamId: profile.linear_config?.team_id ?? '',
    linearApiKey: profile.linear_config?.api_key ?? '',
    linearDefaultAssigneeId: profile.linear_config?.default_assignee_id ?? '',
    linearDefaultBugLabelIds: profile.linear_config?.default_bug_label_ids.join(', ') ?? '',
    linearDefaultFeatureLabelIds: profile.linear_config?.default_feature_label_ids.join(', ') ?? '',
    linearDefaultStateId: profile.linear_config?.default_state_id ?? '',
    titleBugPrefix: profile.title_conventions?.bug_prefix ?? '',
    titleFeaturePrefix: profile.title_conventions?.feature_prefix ?? '',
    areaCategories: profile.area_categories.map(c => ({
      code: c.code,
      name: c.name,
      description: c.description ?? '',
    })),
    customFields: profile.custom_fields.map(f => ({
      key: f.key,
      label: f.label,
      field_type: f.field_type,
      default_value: f.default_value ?? '',
      required: f.required,
    })),
  }
  profileDialogOpen.value = true
}

function addAreaCategory(): void {
  profileForm.value.areaCategories.push({ code: '', name: '', description: '' })
}

function removeAreaCategory(index: number): void {
  profileForm.value.areaCategories.splice(index, 1)
}

function addCustomField(): void {
  profileForm.value.customFields.push({
    key: '',
    label: '',
    field_type: 'text',
    default_value: '',
    required: false,
  })
}

function removeCustomField(index: number): void {
  profileForm.value.customFields.splice(index, 1)
}

function buildProfileFromForm(id: string, now: string): QaProfile {
  const form = profileForm.value

  const hasLinearConfig = form.linearTeamId.trim() || form.linearApiKey.trim()
  const linearConfig = hasLinearConfig
    ? {
        team_id: form.linearTeamId.trim(),
        api_key: form.linearApiKey.trim(),
        default_assignee_id: form.linearDefaultAssigneeId.trim() || null,
        default_bug_label_ids: form.linearDefaultBugLabelIds
          .split(',')
          .map(s => s.trim())
          .filter(s => s.length > 0),
        default_feature_label_ids: form.linearDefaultFeatureLabelIds
          .split(',')
          .map(s => s.trim())
          .filter(s => s.length > 0),
        default_state_id: form.linearDefaultStateId.trim() || null,
      }
    : null

  const hasTitleConventions = form.titleBugPrefix.trim() || form.titleFeaturePrefix.trim()
  const titleConventions = hasTitleConventions
    ? {
        bug_prefix: form.titleBugPrefix.trim(),
        feature_prefix: form.titleFeaturePrefix.trim(),
      }
    : null

  const areaCategories: AreaCategory[] = form.areaCategories
    .filter(c => c.code.trim() || c.name.trim())
    .map(c => ({
      code: c.code.trim(),
      name: c.name.trim(),
      description: c.description.trim() || null,
    }))

  const customFields: CustomMetadataField[] = form.customFields
    .filter(f => f.key.trim() || f.label.trim())
    .map(f => ({
      key: f.key.trim(),
      label: f.label.trim(),
      field_type: f.field_type,
      default_value: f.default_value.trim() || null,
      required: f.required,
    }))

  return {
    id,
    name: form.name.trim(),
    linear_config: linearConfig,
    area_categories: areaCategories,
    custom_fields: customFields,
    title_conventions: titleConventions,
    created_at: now,
    updated_at: now,
  }
}

async function saveProfile(): Promise<void> {
  if (!profileForm.value.name.trim()) return

  try {
    const now = new Date().toISOString()
    if (editingProfile.value) {
      const updated = buildProfileFromForm(editingProfile.value.id, now)
      updated.created_at = editingProfile.value.created_at
      await profileStore.updateProfile(updated)
      $q.notify({ type: 'positive', message: 'Profile updated' })
    } else {
      const id = crypto.randomUUID()
      const created = buildProfileFromForm(id, now)
      await profileStore.createProfile(created)
      $q.notify({ type: 'positive', message: 'Profile created' })
    }
    profileDialogOpen.value = false
  } catch (err) {
    console.error('Failed to save profile:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to save profile',
      caption: err instanceof Error ? err.message : String(err),
    })
  }
}

function confirmDeleteProfile(id: string, name: string): void {
  $q.dialog({
    title: 'Delete Profile',
    message: `Are you sure you want to delete the profile "${name}"? This cannot be undone.`,
    cancel: true,
    persistent: true,
  }).onOk(async () => {
    try {
      await profileStore.deleteProfile(id)
      $q.notify({ type: 'positive', message: 'Profile deleted' })
    } catch (err) {
      console.error('Failed to delete profile:', err)
      $q.notify({
        type: 'negative',
        message: 'Failed to delete profile',
        caption: err instanceof Error ? err.message : String(err),
      })
    }
  })
}

async function onActiveProfileChange(value: string | null): Promise<void> {
  try {
    if (value) {
      await profileStore.setActiveProfile(value)
    } else {
      await profileStore.clearActiveProfile()
    }
  } catch (err) {
    console.error('Failed to change active profile:', err)
    $q.notify({
      type: 'negative',
      message: 'Failed to change active profile',
      caption: err instanceof Error ? err.message : String(err),
    })
  }
}

// Initialize
onMounted(async () => {
  await settingsStore.initialize()
  await loadSettings()
  await checkClaudeStatus()
  await loadTemplateInfo()

  // Load profiles
  try {
    await profileStore.loadProfiles()
  } catch (err) {
    console.warn('Failed to load profiles:', err)
  }

  // Load Linear credentials from ticketing table
  try {
    const creds = await invoke<any>('ticketing_get_credentials')
    if (creds && creds.api_key) {
      localSettings.value.linear_api_key = creds.api_key
      localSettings.value.linear_team_id = creds.team_id || ''
    }
  } catch (err) {
    console.warn('Failed to load Linear credentials:', err)
  }

  // Get app version
  try {
    appVersion.value = await invoke<string>('get_app_version')
  } catch (err) {
    console.warn('Failed to get app version:', err)
    appVersion.value = '1.0.0'
  }

  document.addEventListener('keydown', handleEscapeKey)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscapeKey)
})
</script>

<style scoped>
.settings-container {
  padding-bottom: 2rem;
}
</style>
