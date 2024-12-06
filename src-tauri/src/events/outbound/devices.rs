use super::{send_to_all_plugins, send_to_plugin};

use crate::plugins::info_param::DeviceInfo;

use serde::Serialize;

#[derive(Serialize)]
#[allow(non_snake_case)]
struct DeviceDidConnectEvent {
	event: &'static str,
	device: String,
	deviceInfo: DeviceInfo,
}

#[derive(Serialize)]
struct DeviceDidDisconnectEvent {
	event: &'static str,
	device: String,
}

pub async fn device_did_connect(id: &str, info: DeviceInfo) -> Result<(), anyhow::Error> {
	send_to_all_plugins(&DeviceDidConnectEvent {
		event: "deviceDidConnect",
		device: id.to_owned(),
		deviceInfo: info,
	})
	.await
}

pub async fn device_did_disconnect(id: &str) -> Result<(), anyhow::Error> {
	send_to_all_plugins(&DeviceDidDisconnectEvent {
		event: "deviceDidDisconnect",
		device: id.to_owned(),
	})
	.await
}

#[derive(Serialize)]
struct SetImageEvent {
	event: &'static str,
	device: String,
	position: Option<u8>,
	image: Option<String>,
}

pub async fn update_image(context: crate::shared::Context, image: Option<String>) -> Result<(), anyhow::Error> {
	if let Some(plugin) = crate::devices::DEVICE_NAMESPACES.read().await.get(&context.device[..2]) {
		send_to_plugin(
			plugin,
			&SetImageEvent {
				event: "setImage",
				device: context.device,
				position: Some(context.position),
				image,
			},
		)
		.await?;
	} else if context.device.starts_with("sd-") {
		crate::devices::elgato::update_image(&context, image.as_deref()).await?;
	}

	Ok(())
}

pub async fn clear_screen(device: String) -> Result<(), anyhow::Error> {
	if let Some(plugin) = crate::devices::DEVICE_NAMESPACES.read().await.get(&device[..2]) {
		send_to_plugin(
			plugin,
			&SetImageEvent {
				event: "setImage",
				device,
				position: None,
				image: None,
			},
		)
		.await?;
	} else if device.starts_with("sd-") {
		crate::devices::elgato::clear_screen(&device).await?;
	}

	Ok(())
}

#[derive(Serialize)]
struct SetBrightnessEvent {
	event: &'static str,
	device: String,
	brightness: u8,
}

pub async fn set_brightness(brightness: u8) -> Result<(), anyhow::Error> {
	let devices = crate::devices::DEVICES.read().await;
	let namespaces = crate::devices::DEVICE_NAMESPACES.read().await;
	for device in devices.values() {
		if let Some(plugin) = namespaces.get(&device.id[..2]) {
			send_to_plugin(
				plugin,
				&SetBrightnessEvent {
					event: "setBrightness",
					device: device.id.clone(),
					brightness,
				},
			)
			.await?;
		}
	}
	crate::devices::elgato::set_brightness(brightness).await;

	Ok(())
}
