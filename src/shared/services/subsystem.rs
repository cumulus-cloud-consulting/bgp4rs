// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use async_trait::async_trait;

/// Abstract trait boundary implemented by subsystems which required additional management
/// from the main server task
#[async_trait]
pub trait Subsystem {
    /// Execute any addition tasks required to stop this subsystem.
    async fn stop(&self) -> ();

    /// Get the subsystem name, for example for logging purposes
    fn name(&self) -> &'static str;
}
