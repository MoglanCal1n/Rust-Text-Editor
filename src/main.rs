use iced::{Theme, Element, Application, Settings, Length, Command, Font, Subscription};
use iced::widget::{button, container, text_editor, text, column, row, horizontal_space, tooltip, pick_list};
use iced::executor;
use iced::theme;
use iced::highlighter::{self, Highlighter};
use iced::keyboard;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::path::PathBuf;


fn main() -> iced::Result{
    Editor::run(Settings {
	default_font: Font::MONOSPACE,
	fonts: vec![include_bytes!("../fonts/editor-icons.ttf")
		.as_slice()
		.into()],
	..Settings::default()
     }) 

    
}

struct Editor{
	path: Option<PathBuf>,
	content: text_editor::Content,
	error: Option<Error>,
	theme: highlighter::Theme,
	has_changed: bool,
}

#[derive(Debug, Clone)]
enum Message {
	Edit(text_editor::Action),
	Open,
	New, 
	Save,
	FileSaved(Result<PathBuf, Error>),
	FileOpened(Result<(PathBuf, Arc<String>), Error>),
	ThemeSelected(highlighter::Theme),
}

impl Application for Editor{
	type Message = Message; // User interactions that our application can handle -> Any event that can change the state of our application
	
	type Theme = Theme;
	
	type Executor = executor::Default;
	
	type Flags = ();


	// The new method is a way to initialize our state. This is the way we well iced the state we want our application to be when it starts.
	fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
		(	
			Self {
				path: None,
				content: text_editor::Content::new(),
				error: None,
				theme: highlighter::Theme::SolarizedDark,
				has_changed: true,
			}, 
			Command::perform(
				load_file(default_file()),
				Message::FileOpened,
			),
		)
	}


	// The title of our application
	fn title(&self) -> String {
		String::from("Iced Editor::Calin")
	}


	// Where we handle the events
	fn update(&mut self, message: Self::Message) -> Command<Message>{
		match message {
			Message::Edit(action) => {

				self.has_changed = self.has_changed || action.is_edit();

				self.error = None;

				self.content.edit(action);

				Command::none()
			}

			Message::Open => {
				Command::perform(pick_file(), Message::FileOpened)
			}
				
			Message::New => {
				self.path = None;
				self.content = text_editor::Content::new();

				self.has_changed = true;

				Command::none()
			}
			
			Message::Save => {
				let text = self.content.text();

				Command::perform(save_file(self.path.clone(), text), Message::FileSaved)
			}
	
			Message::FileSaved(Ok(path)) => {
				self.path = Some(path);
				self.has_changed = false;				

				Command::none()
			}

			Message::FileSaved(Err(error)) => {
				self.error = Some(error);

				Command::none()
			}

			Message::FileOpened(Ok((path, content))) => {
				self.path = Some(path);

				self.content = text_editor::Content::with(&content);

				self.has_changed = false;

				Command::none()
			}

			Message::FileOpened(Err(error)) => {
				self.error = Some(error);

				Command::none()
			}

			Message::ThemeSelected(theme) => {
				self.theme = theme;

				Command::none()
			}
		}

	}


	fn subscription(&self) -> Subscription<Message> {
		keyboard::on_key_press(|key_code, modifiers|{
			match key_code {
				keyboard::KeyCode::S if modifiers.command() => Some(Message::Save),
				keyboard::KeyCode::N if modifiers.command() => Some(Message::New),
				keyboard::KeyCode::O if modifiers.command() => Some(Message::Open),
				_=> None,
			}
		})
	}


	// The way to display our events
	fn view(&self) -> Element<'_, Message> {

		let controls = row![
					action(new_icon(), "[Create new file]", Some(Message::New)),
					action(open_icon(), "[Open a file]", Some(Message::Open)),
					action(save_icon(), "[Save the file]", self.has_changed.then_some(Message::Save)),
					horizontal_space(Length::Fill),
					pick_list(highlighter::Theme::ALL, Some(self.theme), Message::ThemeSelected)				
		].spacing(10);

		let input = text_editor(&self.content)
				.on_edit(Message::Edit)
				.highlight::<Highlighter>(
					highlighter::Settings {
						theme : self.theme,
						extension: self
							.path
							.as_ref()
							.and_then(|path| path.extension()?.to_str())
							.unwrap_or("rs")
							.to_string(),
					},
					|highlight, _theme| highlight.to_format(),
				);

		let status_bar = {
		
        	        let status = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
				text(error.to_string())
			} else	{ 
				match self.path.as_deref().and_then(Path::to_str) {
	                        	Some(path) => text(path).size(14),
                        		None => text("New File"), 
				}
                	};

                	let position = {
                        	let (line, column) = self.content.cursor_position();

	                        text(format!("{}:{}", line + 1, column + 1))
        	        };
	

			row![status, horizontal_space(Length::Fill), position]

		};

		container(column![controls, input, status_bar].spacing(10))
			.padding(10)
			.into()
	}

	fn theme(&self) -> Theme {
		if self.theme.is_dark(){
			Theme::Dark
		} else {
			Theme::Light 
		}
	}
}

fn action<'a>(content: Element<'a, Message>, label: &str ,on_press: Option<Message>) -> Element<'a, Message> {
	let is_disabled = on_press.is_none();

	tooltip(
		button(container(content).width(30).center_x())
			.on_press_maybe(on_press)
			.padding([5, 10]).style(
				if is_disabled{
					theme::Button::Secondary
				} else {
					theme::Button::Primary
				}
			), 
		label, 
		tooltip::Position::FollowCursor
	)
	.style(theme::Container::Box)
	.into()
}

fn new_icon<'a>() -> Element<'a, Message> {
	icon('\u{E800}')
}

fn save_icon<'a>() -> Element<'a, Message> {
        icon('\u{E801}')
}

fn open_icon<'a>() -> Element<'a, Message> {
        icon('\u{F115}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Message> {
	const ICON_FONT: Font = Font::with_name("editor-icons");

	text(codepoint).font(ICON_FONT).into()
 }

fn default_file() -> PathBuf {
	 PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}



async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
	let handle = rfd::AsyncFileDialog::new()
		.set_title("[Choose File]")
		.pick_file()
		.await
		.ok_or(Error::DialogClosed)?;
	load_file(handle.path().to_owned()).await
}


async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {

	let contents = tokio::fs::read_to_string(&path)
		.await
		.map(Arc::new)
		.map_err(|error| error.kind())
		.map_err(Error::IOFailed)?;

	Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, text:String) -> Result<PathBuf, Error> {
	let path = if let Some(path) = path {
		path
	} else {
		rfd::AsyncFileDialog::new()
			.set_title("Choose a file name...")
			.save_file()
			.await
			.ok_or(Error::DialogClosed)
			.map(|handle| handle.path().to_owned())? 
	};

	tokio::fs::write(&path, text)
		.await
		.map_err(|error| Error::IOFailed(error.kind()))?;
	
	Ok(path)
} 


#[derive(Debug, Clone)]
enum Error {
	DialogClosed,
	IOFailed(io::ErrorKind)
}
 
