import os
import base64
import streamlit as st
from io import BytesIO
from python_src.StylistAPIComponent import Gender, StylistAPIComponent

st.set_page_config(page_title="Face-Based Clothes Recommendation", layout="wide")

API_BASE_URL = "http://localhost:9500"  # Adjust if needed
api = StylistAPIComponent(API_BASE_URL)

def main():
    st.title("Face-Based Clothes Recommendation")

    # Upload Clothes Section
    st.header("Upload Clothes")
    uploaded_clothes = st.file_uploader("Choose clothes images", type=['png', 'jpg', 'jpeg'], accept_multiple_files=True)
    if uploaded_clothes:
        for file in uploaded_clothes:
            temp_path = f"temp_clothes_{file.name}"
            with open(temp_path, "wb") as f:
                f.write(file.getbuffer())
            name = os.path.splitext(file.name)[0]
            try:
                api.upload_clothes(name, Gender.MALE, temp_path)
                st.success(f"Uploaded {name}")
            except Exception as e:
                st.error(f"Error uploading {name}: {str(e)}")
            finally:
                if os.path.exists(temp_path):
                    os.remove(temp_path)

    # Upload Face Image Section
    st.header("Upload Face Image")
    face_file = st.file_uploader("Upload a face image", type=['png', 'jpg', 'jpeg'])
    top_n = st.slider("Number of items to recommend", 1, 10, 5)

    # Compute Similarities Button
    if st.button("Compute Similarities"):
        if face_file is None:
            st.error("Please upload a face image first.")
        else:
            temp_path_face = f"temp_face_{face_file.name}"
            with open(temp_path_face, "wb") as f:
                f.write(face_file.getbuffer())
                
            st.subheader("Your face image:")
            st.image(face_file, width=300)

            try:
                results = api.calculate_similarity(temp_path_face, top_n)
                st.subheader("Recommended Clothes (ordered by score):")
                recommended = results.get('data', [])
                if recommended:
                    cols = st.columns(3)
                    for idx, item in enumerate(recommended):
                        with cols[idx % 3]:
                            st.write(f"**{item['data_entry']['name']}**")
                            desc = item.get('data_entry', {}).get('descriptions', [])
                            st.write("Descriptions: " + ", ".join(desc) if desc else "No description")
                            st.write(f"Score: {item['score']}")
                            
                            image_data = item.get('data_entry', {}).get('image', '')
                            if image_data:
                                image_bytes = base64.b64decode(image_data)
                                st.image(BytesIO(image_bytes), use_column_width=True)
                else:
                    st.info("No recommendations found.")
            except Exception as e:
                st.error(f"Error finding recommendations: {str(e)}")
            finally:
                if os.path.exists(temp_path_face):
                    os.remove(temp_path_face)

if __name__ == "__main__":
    main()